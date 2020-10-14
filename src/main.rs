use std::env;
use std::io;
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

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("error: {}", msg);
            process::exit(1);
        }
    };

    println!("Hello, world!");
}
