use crate::{
    char_stream::CharStream, errors::SynError, source_file::SourceFile, span::*, tokens::*,
};

pub struct Lexer {
    chars: CharStream,
    tokens: Vec<Token>,
    errors: Vec<SynError>,
    start_pos: Pos,
}

impl Lexer {
    pub fn from_src(src: SourceFile) -> Self {
        Self {
            chars: CharStream::new(src),
            tokens: Vec::new(),
            errors: Vec::new(),
            start_pos: Pos::new(0, 1, 0),
        }
    }

    pub fn resolve(mut self) -> (Vec<Token>, Vec<SynError>) {
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
                            _ if PUNCTUATOR_LEN1_TABLE.contains(&ch_ahead) => {
                                state = 5; // -> 5: punctuator
                                continue 'dfa;
                            }
                            _ => {
                                let ch = self.chars.consume1();
                                self.start_pos = self.chars.pos();
                                self.errors.push(self.error_unexpected_char(ch));
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
                            self.start_pos = self.chars.pos();
                            self.errors.push(self.error_unexpected_char(ch));
                        }
                    },
                },
                // 2: directive
                2 => {
                    match self.expect_directive() {
                        Ok(t) => self.tokens.push(Token::Directive(t)),
                        Err(e) => self.errors.push(e),
                    }
                    state = 0; // -> 0: line start
                    continue 'dfa;
                }
                // 3: comment
                3 => {
                    match self.expect_comment() {
                        Ok(()) => {}
                        Err(e) => self.errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 4: ident
                4 => {
                    match self.expect_ident() {
                        Ok(t) => {
                            if t.is_keyword() {
                                self.tokens.push(Token::Keyword(Keyword {
                                    value: t.value,
                                    span: t.span,
                                }))
                            } else {
                                self.tokens.push(Token::Identifier(t))
                            }
                        }
                        Err(e) => self.errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 5: punctuator
                5 => {
                    match self.expect_punctuator() {
                        Ok(t) => {
                            if t.is_operator() {
                                self.tokens.push(Token::Operator(Operator {
                                    literal: t.literal,
                                    span: t.span,
                                }))
                            } else {
                                self.tokens.push(Token::Punctuator(t))
                            }
                        }
                        Err(e) => self.errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 6: string literal
                6 => {
                    match self.expect_string_literal() {
                        Ok(t) => self.tokens.push(Token::StringLiteral(t)),
                        Err(e) => self.errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 7: constant
                7 => {
                    match self.expect_constant() {
                        Ok(t) => self.tokens.push(Token::Constant(t)),
                        Err(e) => self.errors.push(e),
                    }
                    state = 1; // -> 1: line body
                    continue 'dfa;
                }
                // 255: end
                255 => return (self.tokens, self.errors),
                _ => unreachable!(),
            }
        }
    }
}

impl Lexer {
    #[must_use]
    fn emit_span(&self) -> Span {
        let end_pos = self.chars.pos().add1();
        let start_pos = self.start_pos;

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
    fn emit_error(&self, msg: String) -> SynError {
        SynError {
            span: self.emit_span(),
            msg,
        }
    }

    #[must_use]
    fn save_start_pos<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let start_pos = self.start_pos;
        let ans = f(self);
        self.start_pos = start_pos;
        ans
    }

    fn error_unexpected_char(&self, ch: char) -> SynError {
        self.emit_error(format!("unexpected char: {:?}", ch))
    }

    fn error_expected(&self, which: &str) -> SynError {
        self.emit_error(format!("expected {}", which))
    }
}

impl Lexer {
    fn expect_ident(&mut self) -> Result<Identifier, SynError> {
        let mut literal: String = match self.chars.next() {
            None => return Err(self.error_expected("identifier")),
            Some(ch) => match ch {
                'A'..='Z' | 'a'..='z' | '_' => {
                    self.start_pos = self.chars.pos();
                    ch.into()
                }
                _ => return Err(self.error_unexpected_char(ch)),
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
            span: self.emit_span(),
        })
    }

    fn expect_directive(&mut self) -> Result<Directive, SynError> {
        match self.chars.next() {
            None => return Err(self.error_expected("directive")),
            Some(ch) => match ch {
                '#' => self.start_pos = self.chars.pos(),
                _ => return Err(self.error_unexpected_char(ch)),
            },
        };

        let ident = self.save_start_pos(|this| this.expect_ident())?;

        match self.chars.next() {
            None | Some('\n') => {
                return Ok(Directive {
                    name: ident.value,
                    args: "".into(),
                    span: self.emit_span(),
                })
            }
            Some(ch) => match ch {
                ' ' | '\r' | '\t' | '\x0C' => {
                    // do nothing
                }
                _ => return Err(self.error_unexpected_char(ch)),
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
            span: self.emit_span(),
        })
    }

    fn expect_comment(&mut self) -> Result<(), SynError> {
        match self.chars.next() {
            None => return Err(self.error_expected("comment")),
            Some(ch) => match ch {
                '/' => self.start_pos = self.chars.pos(),
                _ => return Err(self.error_unexpected_char(ch)),
            },
        }

        let is_line_comment = match self.chars.next() {
            None => return Err(self.emit_error("expected comment, found '/'".into())),
            Some(ch) => match ch {
                '/' => true,
                '*' => false,
                _ => return Err(self.error_unexpected_char(ch)),
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
                    (None, _) | (_, None) => return Err(self.emit_error("unclosed comment".into())),
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
        match self.chars.next() {
            None => return Err(self.error_expected("string literal")),
            Some(ch) => match ch {
                '"' => self.start_pos = self.chars.pos(),
                _ => return Err(self.error_unexpected_char(ch)),
            },
        }

        let mut literal = String::new();
        loop {
            match self.chars.next() {
                None => return Err(self.emit_error("unclosed string literal".into())),
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
                '\n' => return Err(self.emit_error("unclosed string literal".into())),
                '\\' => match literal_chars.next() {
                    None => return Err(self.error_expected("escape sequence")),
                    Some(ch) => {
                        match SIMPLE_ESCAPE_SEQUENCE_TABLE
                            .iter()
                            .copied()
                            .find(|&(c, _)| c == ch)
                        {
                            None => {
                                return Err(self.emit_error(format!(
                                    "invalid escape sequence: {:?}",
                                    format!("\\{:?}", ch)
                                )))
                            }
                            Some((_, v)) => value.push(v),
                        }
                    }
                },
                '\u{0}'..='\u{255}' => value.push(ch),
                _ => return Err(self.emit_error("non-ascii string literal".into())),
            }
        }

        Ok(StringLiteral {
            value,
            span: self.emit_span(),
        })
    }

    fn expect_punctuator(&mut self) -> Result<Punctuator, SynError> {
        let ch = self.chars.next();
        self.start_pos = self.chars.pos();
        let ch_ahead = self.chars.peek();

        let (ch1, ch2) = match (ch, ch_ahead) {
            (None, _) => return Err(self.error_expected("punctuator")),
            (Some(ch), None) => {
                if PUNCTUATOR_LEN1_TABLE.contains(&ch) {
                    return Ok(Punctuator {
                        literal: ch.into(),
                        span: self.emit_span(),
                    });
                } else {
                    return Err(self.error_unexpected_char(ch));
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
                        span: self.emit_span(),
                    });
                }
                None | Some(_) => {
                    return Ok(Punctuator {
                        literal: format!("{}{}", ch1, ch2),
                        span: self.emit_span(),
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
                    span: self.emit_span(),
                });
            }
        }

        if PUNCTUATOR_LEN1_TABLE.contains(&ch1) {
            return Ok(Punctuator {
                literal: ch1.into(),
                span: self.emit_span(),
            });
        }

        self.chars.consume1();
        Err(self.error_unexpected_char(ch2))
    }

    fn expect_constant(&mut self) -> Result<Constant, SynError> {
        let ch_leading = match self.chars.next() {
            None => return Err(self.error_expected("constant")),
            Some(ch) => ch,
        };
        self.start_pos = self.chars.pos();

        match ch_leading {
            '\'' => {
                let mut literal = String::new();
                loop {
                    match self.chars.next() {
                        None => return Err(self.emit_error("unclosed char constant".into())),
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
                    None => return Err(self.emit_error("empty char constant".into())),
                    Some(ch) => match ch {
                        '\\' => match literal_chars.next() {
                            None => return Err(self.error_expected("escape sequence")),
                            Some(ch) => {
                                match SIMPLE_ESCAPE_SEQUENCE_TABLE
                                    .iter()
                                    .copied()
                                    .find(|&(c, _)| c == ch)
                                {
                                    None => {
                                        return Err(self.emit_error(format!(
                                            "invalid escape sequence: {:?}",
                                            format!("\\{:?}", ch)
                                        )))
                                    }
                                    Some((_, v)) => value = v,
                                }
                            }
                        },
                        '\u{0}'..='\u{255}' => value = ch,
                        _ => return Err(self.emit_error("non-ascii char constant".into())),
                    },
                }
                if literal_chars.next().is_some() {
                    return Err(self.emit_error("multiple chars in char constant".into()));
                }
                let token = CharConstant {
                    value,
                    span: self.emit_span(),
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
                let span = self.emit_span();
                if has_dot {
                    FloatConstant::validate(literal, span)
                        .map_err(|msg| self.emit_error(msg))
                        .map(Constant::Float)
                } else {
                    IntegerConstant::validate(literal, span)
                        .map_err(|msg| self.emit_error(msg))
                        .map(Constant::Int)
                }
            }
            _ => Err(self.error_unexpected_char(ch_leading)),
        }
    }
}
