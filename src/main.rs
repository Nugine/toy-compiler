pub mod char_stream;
pub mod errors;
pub mod lexer;
pub mod source_file;
pub mod span;
pub mod tokens;
pub mod utils;

use std::env;
use std::fmt;
use std::fs;
use std::process;

struct Args {
    src_path: String,
}

fn parse_args() -> Result<Args, String> {
    let mut args_iter = env::args();
    args_iter.next();

    let src_path = args_iter
        .next()
        .ok_or_else(|| "missing argument: source path")?;

    if args_iter.next().is_some() {
        return Err("too many arguments".into());
    }

    Ok(Args { src_path })
}

fn exit_on_error<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1)
        }
    }
}

use crate::errors::SynError;
use crate::lexer::Lexer;
use crate::source_file::SourceFile;
use crate::tokens::Token;
use crate::utils::number_width;

fn print_token(token: &Token) {
    match token {
        Token::Identifier(ident) => {
            println!("(Identifier, {:?})", ident.value);
        }
        Token::Keyword(keyword) => {
            println!("(Keyword, {:?})", keyword.value);
        }
        Token::Constant(constant) => match constant {
            tokens::Constant::Int(int) => {
                println!("(IntegerConstant, {:?})", int.literal);
            }
            tokens::Constant::Float(float) => {
                println!("(FloatConstant, {:?})", float.literal);
            }
            tokens::Constant::Char(ch) => {
                println!("(CharConstant, {:?})", ch.value as char);
            }
        },
        Token::StringLiteral(s) => {
            println!("(StringLiteral, {:?})", s.value);
        }
        Token::Punctuator(punc) => {
            println!("(Punctuator, {:?})", punc.literal);
        }
        Token::Operator(op) => {
            println!("(Operator, {:?})", op.literal);
        }
        Token::Directive(directive) => {
            println!("(Directive, {:?}, {:?})", directive.name, directive.args);
        }
    }
}

fn print_token_span(token: &Token, src_lines: &[Vec<char>]) {
    let span = token.span();
    let start_lc = span.lc_range.start;
    let end_lc = span.lc_range.end;
    if start_lc.line == end_lc.line {
        let line_str = src_lines[&start_lc.line - 1]
            .iter()
            .copied()
            .collect::<String>();

        let indicator = line_str
            .chars()
            .enumerate()
            .map(|(i, ch)| {
                let col = i + 1;
                if col >= start_lc.column && col < end_lc.column {
                    '^'
                } else if ch == '\t' {
                    '\t'
                } else {
                    ' '
                }
            })
            .collect::<String>();

        println!(
            " --> {}:{}:{}",
            span.file_path, start_lc.line, start_lc.column
        );
        println!("{}", line_str);
        println!("{}", indicator);
    } else {
        // todo?
    }
}

fn eprint_error_span(error: &SynError, src_lines: &[Vec<char>]) {
    let start_lc = error.span.lc_range.start;
    let end_lc = error.span.lc_range.end;

    if start_lc.line == end_lc.line {
        let line_str = src_lines[&start_lc.line - 1]
            .iter()
            .copied()
            .collect::<String>();

        let indicator = line_str
            .chars()
            .enumerate()
            .map(|(i, ch)| {
                let col = i + 1;
                if col >= start_lc.column && col < end_lc.column {
                    '^'
                } else if ch == '\t' {
                    '\t'
                } else {
                    ' '
                }
            })
            .collect::<String>();

        eprintln!(
            " --> {}:{}:{}",
            error.span.file_path, start_lc.line, start_lc.column
        );
        eprintln!("{}", line_str);
        eprintln!("{}", indicator);
    } else {
        let start_line_str = src_lines[start_lc.line - 1]
            .iter()
            .copied()
            .collect::<String>();
        let end_line_str = src_lines[end_lc.line - 1]
            .iter()
            .copied()
            .collect::<String>();

        let start_indicator = start_line_str
            .chars()
            .enumerate()
            .map(|(i, ch)| {
                let col = i + 1;
                if col >= start_lc.column {
                    '^'
                } else if ch == '\t' {
                    '\t'
                } else {
                    ' '
                }
            })
            .collect::<String>();

        let end_indicator = end_line_str
            .chars()
            .enumerate()
            .map(|(i, ch)| {
                let col = i + 1;
                if col < end_lc.column {
                    '^'
                } else if ch == '\t' {
                    '\t'
                } else {
                    ' '
                }
            })
            .collect::<String>();

        eprintln!(
            " --> starts at {}:{}:{}",
            error.span.file_path, start_lc.line, start_lc.column
        );
        eprintln!("{}", start_line_str);
        eprintln!("{}", start_indicator);
        eprintln!(
            " --> ends at {}:{}:{}",
            error.span.file_path, end_lc.line, end_lc.column
        );
        eprintln!("{}", end_line_str);
        eprintln!("{}", end_indicator);
    }
}

fn main() {
    let args = exit_on_error(parse_args());

    let source_file = {
        let content = exit_on_error(fs::read_to_string(&args.src_path));
        SourceFile::new(&content, &args.src_path)
    };

    let src_lines = source_file.generate_lines();

    {
        // print lines with lineno
        let lineno_width = number_width(src_lines.len()).max(2);
        for (idx, line) in src_lines.iter().enumerate() {
            let lineno = idx + 1;
            let line = line.iter().copied().collect::<String>();
            println!("{:>width$}| {}", lineno, line, width = lineno_width);
        }
        println!();
    }

    let lexer = Lexer::from_src(source_file);
    let (tokens, errors) = lexer.resolve();

    for token in &tokens {
        print_token(token);
        print_token_span(token, &src_lines);
        println!();
    }

    if !errors.is_empty() {
        eprintln!();
        for error in &errors {
            eprintln!("error: {}", error.msg);
            eprint_error_span(error, &src_lines);
            eprintln!();
        }
        process::exit(1);
    }
}
