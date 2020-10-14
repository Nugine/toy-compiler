mod char_stream;
mod errors;
mod source_file;
mod span;

use crate::source_file::SourceFile;

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

fn number_width(n: usize) -> usize {
    if n < 10 {
        1
    } else {
        ((n as f64).log10() as usize) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_number_width() {
        assert_eq!(number_width(0), 1);
        assert_eq!(number_width(9), 1);
        assert_eq!(number_width(10), 2);
        assert_eq!(number_width(100), 3);
        assert_eq!(number_width(1000), 4);
        assert_eq!(number_width(10000), 5);
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
    }

    println!("Hello, world!");
}
