use crate::{
    char_stream::CharStream, errors::SynError, source_file::SourceFile, span::*, tokens::*,
};

pub struct Lexer {
    chars: CharStream,
}

impl Lexer {
    pub fn from_src(src: SourceFile) -> Self {
        Self {
            chars: CharStream::new(src),
        }
    }

    pub fn resolve(mut self) -> (Vec<Token>, Vec<SynError>) {
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors: Vec<SynError> = Vec::new();
        let mut start_pos;

        let mut state = 0;

        'dfa: loop {
            match state {
                // 0: line start
                0 => match self.chars.peek() {
                    None => {
                        state = 255; // -> 255: end
                        continue 'dfa;
                    }
                    Some(ch_ahead) => {
                        match ch_ahead {
                            '\n' | '\t' | '\x0C' | '\r' | ' ' => {
                                self.chars.consume1();
                                state = 0; // -> 0: line start
                                continue 'dfa;
                            }
                            '#' => {
                                state = 2; // -> 2: directive
                                continue 'dfa;
                            }
                            '/' => match self.chars.peek2() {
                                None => {
                                    state = 5; // -> 5: punctuator
                                    continue 'dfa;
                                }
                                Some(ch_ahead2) => match ch_ahead2 {
                                    '/' | '*' => {
                                        state = 3; // -> 3: comment
                                        continue 'dfa;
                                    }
                                    _ => {
                                        state = 5; // -> 5: punctuator
                                        continue 'dfa;
                                    }
                                },
                            },
                            'A'..='Z' | 'a'..='z' | '_' => {
                                state = 4; // -> 4: ident
                                continue 'dfa;
                            }
                            '"' => {
                                state = 6; // -> 6: string literal
                                continue 'dfa;
                            }
                            '0'..='9' => {
                                state = 7; // -> 7: constant
                                continue 'dfa;
                            }
                            _ if PUNCTUATOR_LEN1_TABLE.contains(&ch_ahead) => {
                                state = 5; // -> 5: punctuator
                                continue 'dfa;
                            }
                            _ => {
                                let ch = self.chars.consume1();
                                start_pos = self.chars.pos();
                                errors.push(self.error_unexpected_char(ch, start_pos));
                            }
                        }
                    }
                },
                // 1: line body
                1 => match self.chars.peek() {
                    None => {
                        state = 255; // -> 255: end
                        continue 'dfa;
                    }
                    Some(ch_ahead) => match ch_ahead {
                        '\n' => {
                            self.chars.consume1();
                            state = 0; // -> 0: line start
                            continue 'dfa;
                        }
                        ' ' | '\r' | '\t' | '\x0C' => {
                            self.chars.consume1();
                            state = 1; // -> 1: line body
                            continue 'dfa;
                        }
                        '/' => match self.chars.peek() {
                            None => {
                                state = 5; // -> 5: punctuator
                                continue 'dfa;
                            }
                            Some(ch_ahead) => match ch_ahead {
                                '/' | '*' => {
                                    state = 3; // -> 3: comment
                                    continue 'dfa;
                                }
                                _ => {
                                    state = 5; // -> 5: punctuator
                                    continue 'dfa;
                                }
                            },
                        },
                        'A'..='Z' | 'a'..='z' | '_' => {
                            state = 4; // -> 4: ident
                            continue 'dfa;
                        }
                        '.' => match self.chars.peek() {
                            None => {
                                state = 5;
                                continue 'dfa;
                            }
                            Some(ch_ahead) => match ch_ahead {
                                '0'..='9' => {
                                    state = 7;
                                    continue 'dfa;
                                }
                                _ => {
                                    state = 5;
                                    continue 'dfa;
                                }
                            },
                        },
                        '"' => {
                            state = 6; // -> 6: string literal
                            continue 'dfa;
                        }
                        '0'..='9' | '\'' => {
                            state = 7; // -> 7: constant
                            continue 'dfa;
                        }
                        _ if PUNCTUATOR_LEN1_TABLE.contains(&ch_ahead) => {
                            state = 5; // -> 5: punctuator
                            continue 'dfa;
                        }
                        _ => {
                            let ch = self.chars.consume1();
                            start_pos = self.chars.pos();
                            errors.push(self.error_unexpected_char(ch, start_pos));
                        }
                    },
                },
                // 2: directive
                2 => {
                    match self.expect_directive() {
                        Ok(t) => tokens.push(Token::Directive(t)),
                        Err(e) => errors.push(e),
                    }
                    state = 0; // -> 0: line start
                    continue 'dfa;
                }
                // 3: comment
                3 => {
                    match self.expect_comment() {
                        Ok(()) => {}
                        Err(e) => errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 4: ident
                4 => {
                    match self.expect_ident() {
                        Ok(t) => {
                            if t.is_keyword() {
                                tokens.push(Token::Keyword(Keyword {
                                    value: t.value,
                                    span: t.span,
                                }))
                            } else {
                                tokens.push(Token::Identifier(t))
                            }
                        }
                        Err(e) => errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 5: punctuator
                5 => {
                    match self.expect_punctuator() {
                        Ok(t) => {
                            if t.is_operator() {
                                tokens.push(Token::Operator(Operator {
                                    literal: t.literal,
                                    span: t.span,
                                }))
                            } else {
                                tokens.push(Token::Punctuator(t))
                            }
                        }
                        Err(e) => errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 6: string literal
                6 => {
                    match self.expect_string_literal() {
                        Ok(t) => tokens.push(Token::StringLiteral(t)),
                        Err(e) => errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 7: constant
                7 => {
                    match self.expect_constant() {
                        Ok(t) => tokens.push(Token::Constant(t)),
                        Err(e) => errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 255: end
                255 => return (tokens, errors),
                _ => unreachable!(),
            }
        }
    }
}

impl Lexer {
    #[must_use]
    fn emit_span(&self, start_pos: Pos) -> Span {
        let end_pos = self.chars.pos().add1();

        let start_lc = LineColumn {
            line: start_pos.lineno,
            column: start_pos.column,
        };

        let end_lc = LineColumn {
            line: end_pos.lineno,
            column: end_pos.column,
        };

        Span {
            byte_range: start_pos.byte_pos..end_pos.byte_pos,
            lc_range: start_lc..end_lc,
            file_path: self.chars.file_path().clone(),
        }
    }

    #[must_use]
    fn emit_error(&self, msg: String, start_pos: Pos) -> SynError {
        SynError {
            span: self.emit_span(start_pos),
            msg,
        }
    }

    fn error_unexpected_char(&self, ch: char, start_pos: Pos) -> SynError {
        self.emit_error(format!("unexpected char: {:?}", ch), start_pos)
    }

    fn error_expected(&self, which: &str, start_pos: Pos) -> SynError {
        self.emit_error(format!("expected {}", which), start_pos)
    }
}

impl Lexer {
    fn expect_ident(&mut self) -> Result<Identifier, SynError> {
        let start_pos = self.chars.pos().add1();

        let mut literal: String = match self.chars.next() {
            None => return Err(self.error_expected("identifier", start_pos)),
            Some(ch) => match ch {
                'A'..='Z' | 'a'..='z' | '_' => ch.into(),
                _ => return Err(self.error_unexpected_char(ch, start_pos)),
            },
        };

        while let Some(ch_ahead) = self.chars.peek() {
            match ch_ahead {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                    let ch = self.chars.consume1();
                    literal.push(ch);
                }
                _ => {
                    break;
                }
            }
        }

        Ok(Identifier {
            value: literal,
            span: self.emit_span(start_pos),
        })
    }

    fn expect_directive(&mut self) -> Result<Directive, SynError> {
        let start_pos = self.chars.pos().add1();

        match self.chars.next() {
            None => return Err(self.error_expected("directive", start_pos)),
            Some(ch) => match ch {
                '#' => {}
                _ => return Err(self.error_unexpected_char(ch, start_pos)),
            },
        };

        let ident = self.expect_ident()?;

        match self.chars.next() {
            None | Some('\n') => {
                return Ok(Directive {
                    name: ident.value,
                    args: "".into(),
                    span: self.emit_span(start_pos),
                })
            }
            Some(ch) => match ch {
                ' ' | '\r' | '\t' | '\x0C' => {
                    // do nothing
                }
                _ => return Err(self.error_unexpected_char(ch, start_pos)),
            },
        }

        let mut args = String::new();
        while let Some(ch) = self.chars.next() {
            match ch {
                '\n' => break,
                '\r' => {
                    if self.chars.peek() == Some('\n') {
                        self.chars.consume1();
                        break;
                    }
                }
                _ => args.push(ch),
            }
        }

        Ok(Directive {
            name: ident.value,
            args,
            span: self.emit_span(start_pos),
        })
    }

    fn expect_comment(&mut self) -> Result<(), SynError> {
        let start_pos = self.chars.pos().add1();

        match self.chars.next() {
            None => return Err(self.error_expected("comment", start_pos)),
            Some(ch) => match ch {
                '/' => {}
                _ => return Err(self.error_unexpected_char(ch, start_pos)),
            },
        }

        let is_line_comment = match self.chars.next() {
            None => return Err(self.emit_error("expected comment, found '/'".into(), start_pos)),
            Some(ch) => match ch {
                '/' => true,
                '*' => false,
                _ => return Err(self.error_unexpected_char(ch, start_pos)),
            },
        };

        if is_line_comment {
            while let Some(ch) = self.chars.next() {
                if ch == '\n' {
                    break;
                }
            }
            Ok(())
        } else {
            loop {
                let ch = self.chars.next();
                let ch_ahead = self.chars.peek();

                match (ch, ch_ahead) {
                    (None, _) | (_, None) => {
                        return Err(self.emit_error("unclosed comment".into(), start_pos))
                    }
                    (Some(ch), Some(ch_ahead)) => {
                        if let ('*', '/') = (ch, ch_ahead) {
                            self.chars.consume1();
                            break;
                        }
                    }
                }
            }
            Ok(())
        }
    }

    fn expect_string_literal(&mut self) -> Result<StringLiteral, SynError> {
        let start_pos = self.chars.pos().add1();

        match self.chars.next() {
            None => return Err(self.error_expected("string literal", start_pos)),
            Some(ch) => match ch {
                '"' => {}
                _ => return Err(self.error_unexpected_char(ch, start_pos)),
            },
        }

        let mut literal = String::new();
        loop {
            match self.chars.next() {
                None => return Err(self.emit_error("unclosed string literal".into(), start_pos)),
                Some(ch) => match ch {
                    '"' | '\n' => break,
                    '\\' => match self.chars.next() {
                        None => break,
                        Some(ch) => literal.push(ch),
                    },

                    _ => literal.push(ch),
                },
            }
        }

        let mut value = String::new();
        let mut literal_chars = literal.chars();
        while let Some(ch) = literal_chars.next() {
            match ch {
                '\n' => return Err(self.emit_error("unclosed string literal".into(), start_pos)),
                '\\' => match literal_chars.next() {
                    None => return Err(self.error_expected("escape sequence", start_pos)),
                    Some(ch) => {
                        match SIMPLE_ESCAPE_SEQUENCE_TABLE
                            .iter()
                            .copied()
                            .find(|&(c, _)| c == ch)
                        {
                            None => {
                                return Err(self.emit_error(
                                    format!("invalid escape sequence: {:?}", format!("\\{:?}", ch)),
                                    start_pos,
                                ))
                            }
                            Some((_, v)) => value.push(v),
                        }
                    }
                },
                '\u{0}'..='\u{255}' => value.push(ch),
                _ => return Err(self.emit_error("non-ascii string literal".into(), start_pos)),
            }
        }

        Ok(StringLiteral {
            value,
            span: self.emit_span(start_pos),
        })
    }

    fn expect_punctuator(&mut self) -> Result<Punctuator, SynError> {
        let mut start_pos = self.chars.pos().add1();

        let ch = self.chars.next();
        let ch_ahead = self.chars.peek();

        let (ch1, ch2) = match (ch, ch_ahead) {
            (None, _) => return Err(self.error_expected("punctuator", start_pos)),
            (Some(ch), None) => {
                if PUNCTUATOR_LEN1_TABLE.contains(&ch) {
                    return Ok(Punctuator {
                        literal: ch.into(),
                        span: self.emit_span(start_pos),
                    });
                } else {
                    return Err(self.error_unexpected_char(ch, start_pos));
                }
            }
            (Some(ch), Some(ch_ahead)) => (ch, ch_ahead),
        };

        if let ('<', '<') | ('>', '>') = (ch1, ch2) {
            self.chars.consume1();
            match self.chars.peek() {
                Some('=') => {
                    self.chars.consume1();
                    return Ok(Punctuator {
                        literal: format!("{}{}=", ch1, ch2),
                        span: self.emit_span(start_pos),
                    });
                }
                None | Some(_) => {
                    return Ok(Punctuator {
                        literal: format!("{}{}", ch1, ch2),
                        span: self.emit_span(start_pos),
                    })
                }
            }
        };

        let punc = format!("{}{}", ch1, ch2);
        for &op in &OPERATOR_TABLE {
            if op.len() == 2 && op == punc {
                self.chars.consume1();
                return Ok(Punctuator {
                    literal: op.into(),
                    span: self.emit_span(start_pos),
                });
            }
        }

        if PUNCTUATOR_LEN1_TABLE.contains(&ch1) {
            return Ok(Punctuator {
                literal: ch1.into(),
                span: self.emit_span(start_pos),
            });
        }

        self.chars.consume1();
        start_pos = self.chars.pos();
        Err(self.error_unexpected_char(ch2, start_pos))
    }

    fn expect_constant(&mut self) -> Result<Constant, SynError> {
        let start_pos = self.chars.pos().add1();

        let ch_leading = match self.chars.next() {
            None => return Err(self.error_expected("constant", start_pos)),
            Some(ch) => ch,
        };

        match ch_leading {
            '\'' => {
                let mut literal = String::new();
                loop {
                    match self.chars.next() {
                        None => {
                            return Err(self.emit_error("unclosed char constant".into(), start_pos))
                        }
                        Some(ch) => match ch {
                            '\'' => break,
                            '\\' => match self.chars.next() {
                                None => break,
                                Some(ch) => literal.push(ch),
                            },
                            _ => literal.push(ch),
                        },
                    };
                }
                let value;
                let mut literal_chars = literal.chars();
                match literal_chars.next() {
                    None => return Err(self.emit_error("empty char constant".into(), start_pos)),
                    Some(ch) => match ch {
                        '\\' => match literal_chars.next() {
                            None => return Err(self.error_expected("escape sequence", start_pos)),
                            Some(ch) => {
                                match SIMPLE_ESCAPE_SEQUENCE_TABLE
                                    .iter()
                                    .copied()
                                    .find(|&(c, _)| c == ch)
                                {
                                    None => {
                                        return Err(self.emit_error(
                                            format!(
                                                "invalid escape sequence: {:?}",
                                                format!("\\{:?}", ch)
                                            ),
                                            start_pos,
                                        ))
                                    }
                                    Some((_, v)) => value = v,
                                }
                            }
                        },
                        '\u{0}'..='\u{255}' => value = ch,
                        _ => {
                            return Err(self.emit_error("non-ascii char constant".into(), start_pos))
                        }
                    },
                }
                if literal_chars.next().is_some() {
                    return Err(
                        self.emit_error("multiple chars in char constant".into(), start_pos)
                    );
                }
                let token = CharConstant {
                    value,
                    span: self.emit_span(start_pos),
                };
                Ok(Constant::Char(token))
            }

            '.' | '0'..='9' => {
                let mut literal: String = ch_leading.into();

                let mut has_dot = false;
                while let Some(ch_ahead) = self.chars.peek() {
                    match ch_ahead {
                        '0'..='9' | 'A'..='Z' | 'a'..='z' => {
                            literal.push(self.chars.consume1());
                        }
                        '.' => {
                            has_dot = true;
                            literal.push(self.chars.consume1());
                            while let Some(ch_ahead) = self.chars.peek() {
                                match ch_ahead {
                                    '0'..='9' | 'A'..='Z' | 'a'..='z' => {
                                        literal.push(self.chars.consume1());
                                    }
                                    _ => break,
                                }
                            }
                            break;
                        }
                        _ => break,
                    }
                }
                let span = self.emit_span(start_pos);
                if has_dot {
                    FloatConstant::validate(literal, span)
                        .map_err(|msg| self.emit_error(msg, start_pos))
                        .map(Constant::Float)
                } else {
                    IntegerConstant::validate(literal, span)
                        .map_err(|msg| self.emit_error(msg, start_pos))
                        .map(Constant::Int)
                }
            }
            _ => Err(self.error_unexpected_char(ch_leading, start_pos)),
        }
    }
}
