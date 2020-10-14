use crate::span::Span;

#[derive(Debug)]
pub enum Token {
    Identifier(Identifier),
    Keyword(Keyword),
    Constant(Constant),
    StringLiteral(StringLiteral),
    Punctuator(Punctuator),
    Operator(Operator),
    Directive(Directive),
}

#[derive(Debug)]
pub struct Identifier {
    pub value: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct Keyword {
    pub value: String,
    pub span: Span,
}

#[derive(Debug)]
pub enum Constant {
    Int(IntegerConstant),
    Float(FloatConstant),
    Char(CharConstant),
}

#[derive(Debug)]
pub struct IntegerConstant {
    pub literal: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct FloatConstant {
    pub literal: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct CharConstant {
    pub value: char,
    pub span: Span,
}

#[derive(Debug)]
pub struct StringLiteral {
    pub value: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct Punctuator {
    pub literal: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct Operator {
    pub literal: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct Directive {
    pub name: String,
    pub args: String,
    pub span: Span,
}

impl Token {
    pub fn span(&self) -> &Span {
        match self {
            Token::Identifier(ident) => &ident.span,
            Token::Keyword(kw) => &kw.span,
            Token::Constant(constant) => match constant {
                Constant::Int(int) => &int.span,
                Constant::Float(float) => &float.span,
                Constant::Char(ch) => &ch.span,
            },
            Token::Operator(op) => &op.span,
            Token::StringLiteral(s) => &s.span,
            Token::Punctuator(p) => &p.span,
            Token::Directive(d) => &d.span,
        }
    }
}

impl Identifier {
    pub fn is_keyword(&self) -> bool {
        KEYWORD_TABLE.iter().any(|&s| s == self.value)
    }
}

impl Punctuator {
    pub fn is_operator(&self) -> bool {
        OPERATOR_TABLE.iter().any(|&s| s == self.literal)
    }
}

pub static KEYWORD_TABLE: [&str; 44] = [
    "alignof",
    "auto",
    "break",
    "case",
    "char",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "typedef",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    "_Alignas",
    "_Atomic",
    "_Bool",
    "_Complex",
    "_Generic",
    "_Imaginary",
    "_Noreturn",
    "_Static_assert",
    "_Thread_local",
];

pub static OPERATOR_TABLE: [&str; 35] = [
    ".", "->", "++", "--", "&", "*", "+", "-", "~", "!", "/", "%", "<<", ">>", "<", ">", "<=",
    ">=", "=", "*=", "/=", "%=", "+=", "-=", "<<=", "==", ">>=", "!=", "&=", "^", "|", "^=", "&&",
    "||", "|=",
];

pub static SIMPLE_ESCAPE_SEQUENCE_TABLE: [(char, char); 12] = [
    ('\'', '\''),
    ('"', '"'),
    ('?', '?'),
    ('\\', '\\'),
    ('a', '\x07'),
    ('b', '\x08'),
    ('f', '\x0C'),
    ('v', '\x0B'),
    ('n', '\n'),
    ('r', '\r'),
    ('t', '\t'),
    ('0', '\0'),
];

pub static INTEGER_SUFFIX_TABLE: [&str; 22] = [
    "u", "U", "l", "L", "ll", "LL", "ul", "uL", "ull", "uLL", "Ul", "UL", "Ull", "ULL", "lu", "Lu",
    "llu", "LLu", "lU", "LU", "llU", "LLU",
];

impl IntegerConstant {
    pub fn validate(literal: String, span: Span) -> Result<Self, String> {
        let mut chars = literal.chars();

        let mut state = 0;

        let mut suffix = String::new();
        let error_invalid_char = |ch: char| format!("invalid char in integer constant: {:?}", ch);

        'dfa: loop {
            match state {
                0 => match chars.next() {
                    None => panic!("empty integer constant"),
                    Some(ch) => match ch {
                        '1'..='9' => {
                            state = 1;
                            continue 'dfa;
                        }
                        '0' => {
                            state = 2;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                1 => match chars.next() {
                    None => {
                        state = 8;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 3;
                            continue 'dfa;
                        }
                        'A'..='Z' | 'a'..='z' => {
                            suffix = ch.into();
                            state = 7;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                2 => match chars.next() {
                    None => {
                        state = 8;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        'x' | 'X' => {
                            state = 4;
                            continue 'dfa;
                        }
                        '0'..='7' => {
                            state = 5;
                            continue 'dfa;
                        }
                        '8'..='9' => {
                            return Err(format!("invalid digit {:?} in octal constant", ch))
                        }
                        'A'..='Z' | 'a'..='z' => {
                            suffix = ch.into();
                            state = 7;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                3 => match chars.next() {
                    None => {
                        state = 8;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 3;
                            continue 'dfa;
                        }
                        'A'..='Z' | 'a'..='z' => {
                            suffix = ch.into();
                            state = 7;
                            continue 'dfa;
                        }
                        _ => return Err(format!("invalid char in decimal constant: {:?}", ch)),
                    },
                },
                4 => match chars.next() {
                    None => return Err("expected hexadecimal digit".into()),
                    Some(ch) => match ch {
                        '0'..='9' | 'A'..='F' | 'a'..='f' => {
                            state = 6;
                            continue 'dfa;
                        }
                        _ => return Err(format!("invalid char in hexadecimal constant: {:?}", ch)),
                    },
                },
                5 => match chars.next() {
                    None => {
                        state = 8;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        '0'..='7' => {
                            state = 5;
                            continue 'dfa;
                        }
                        '8'..='9' => {
                            return Err(format!("invalid digit in octal constant: {:?} ", ch))
                        }
                        'A'..='Z' | 'a'..='z' => {
                            suffix = ch.into();
                            state = 7;
                            continue 'dfa;
                        }
                        _ => return Err(format!("invalid char in octal constant: {:?}", ch)),
                    },
                },
                6 => match chars.next() {
                    None => {
                        state = 8;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        '0'..='9' | 'A'..='F' | 'a'..='f' => {
                            state = 6;
                            continue 'dfa;
                        }
                        'A'..='Z' | 'a'..='z' => {
                            suffix = ch.into();
                            state = 7;
                            continue 'dfa;
                        }
                        _ => return Err(format!("invalid char in hexadecimal constant: {:?}", ch)),
                    },
                },
                7 => match chars.next() {
                    None => {
                        let is_valid_suffix = INTEGER_SUFFIX_TABLE.iter().any(|&s| s == suffix);
                        if is_valid_suffix {
                            state = 8;
                            continue 'dfa;
                        } else {
                            return Err(format!("invalid integer suffix: {:?}", suffix));
                        }
                    }
                    Some(ch) => {
                        suffix.push(ch);
                        state = 7;
                        continue 'dfa;
                    }
                },
                8 => return Ok(IntegerConstant { literal, span }),
                _ => unreachable!(),
            }
        }
    }
}

impl FloatConstant {
    pub fn validate(literal: String, span: Span) -> Result<Self, String> {
        let mut chars = literal.chars().peekable();
        let error_invalid_char = |ch: char| format!("invalid char in float constant: {:?}", ch);

        let mut state = 0;

        'dfa: loop {
            match state {
                0 => match chars.next() {
                    None => panic!("empty float literal"),
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 1;
                            continue 'dfa;
                        }
                        '.' => {
                            state = 2;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                1 => match chars.next() {
                    None => return Err("invalid float constant".into()),
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 1;
                            continue 'dfa;
                        }
                        '.' => {
                            state = 3;
                            continue 'dfa;
                        }
                        'e' | 'E' => {
                            state = 6;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                2 => match chars.next() {
                    None => return Err("expected digit sequence".into()),
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 4;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                3 => match chars.next() {
                    None => {
                        state = 10;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 5;
                            continue 'dfa;
                        }
                        'e' | 'E' => {
                            state = 6;
                            continue 'dfa;
                        }
                        'f' | 'F' | 'l' | 'L' => {
                            state = 9;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                4 => match chars.next() {
                    None => {
                        state = 10;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 4;
                            continue 'dfa;
                        }
                        'e' | 'E' => {
                            state = 6;
                            continue 'dfa;
                        }
                        'f' | 'F' | 'l' | 'L' => {
                            state = 9;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                5 => match chars.next() {
                    None => {
                        state = 10;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 5;
                            continue 'dfa;
                        }
                        'e' | 'E' => {
                            state = 6;
                            continue 'dfa;
                        }
                        'f' | 'F' | 'l' | 'L' => {
                            state = 9;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },

                6 => match chars.next() {
                    None => return Err("expected exponent part".into()),
                    Some(ch) => match ch {
                        '+' | '-' => {
                            state = 7;
                            continue 'dfa;
                        }
                        '0'..='9' => {
                            state = 8;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },

                7 => match chars.next() {
                    None => return Err("expected exponent part".into()),
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 8;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },

                8 => match chars.next() {
                    None => {
                        state = 10;
                        continue 'dfa;
                    }
                    Some(ch) => match ch {
                        '0'..='9' => {
                            state = 8;
                            continue 'dfa;
                        }
                        'f' | 'F' | 'l' | 'L' => {
                            state = 9;
                            continue 'dfa;
                        }
                        _ => return Err(error_invalid_char(ch)),
                    },
                },
                9 => match chars.next() {
                    None => {
                        state = 10;
                        continue 'dfa;
                    }
                    Some(ch) => return Err(error_invalid_char(ch)),
                },
                10 => return Ok(Self { literal, span }),
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::span::LineColumn;

    fn dummy_span() -> Span {
        Span {
            byte_range: 0..0,
            lc_range: LineColumn { line: 0, column: 0 }..LineColumn { line: 0, column: 0 },
            file_path: "<dummy file>".into(),
        }
    }

    #[test]
    fn validate_float() {
        assert!(FloatConstant::validate("1.2e-34".into(), dummy_span()).is_ok());
        assert!(FloatConstant::validate(".2e-34".into(), dummy_span()).is_ok());
        assert!(FloatConstant::validate(".2".into(), dummy_span()).is_ok());
        assert!(FloatConstant::validate("1.".into(), dummy_span()).is_ok());

        assert!(FloatConstant::validate(".".into(), dummy_span()).is_err());
        assert!(FloatConstant::validate("1.e".into(), dummy_span()).is_err());
        assert!(FloatConstant::validate("1.e-".into(), dummy_span()).is_err());
        assert!(FloatConstant::validate("1.e-x".into(), dummy_span()).is_err());
    }

    #[test]
    fn validate_int() {
        assert!(IntegerConstant::validate("0".into(), dummy_span()).is_ok());
        assert!(IntegerConstant::validate("123".into(), dummy_span()).is_ok());
        assert!(IntegerConstant::validate("017".into(), dummy_span()).is_ok());
        assert!(IntegerConstant::validate("0xff".into(), dummy_span()).is_ok());
        assert!(IntegerConstant::validate("0xffL".into(), dummy_span()).is_ok());
        assert!(IntegerConstant::validate("1U".into(), dummy_span()).is_ok());

        assert!(IntegerConstant::validate("0178".into(), dummy_span()).is_err());
        assert!(IntegerConstant::validate("0xgg".into(), dummy_span()).is_err());
        assert!(IntegerConstant::validate(".".into(), dummy_span()).is_err());
        assert!(IntegerConstant::validate("1.0".into(), dummy_span()).is_err());
    }
}
