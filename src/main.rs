use runner::{ErrorKind, Lox};
use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, Write},
    process::ExitCode,
};

mod ast;
mod compile;
mod error;
mod error_reporter;
mod runner;
mod runtime;
mod span;

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match &args[..] {
        [] => repl(),
        [path] => run_file(path),
        _ => incorrect_usage(),
    }
}

fn repl() -> ExitCode {
    let mut lox = Lox::new();
    loop {
        let mut line = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line).unwrap();
        if line.trim_end().is_empty() {
            break;
        }
        let _ = lox.run(line.trim_end());
    }
    ExitCode::from(0)
}

fn run_file(filename: &OsStr) -> ExitCode {
    let contents = fs::read_to_string(filename);
    if let Ok(contents) = contents {
        match Lox::new().run(&contents) {
            Ok(_) => ExitCode::from(0),
            Err(error_kind) => exit_code_from_error_kind(error_kind),
        }
    } else {
        eprintln!("Should have been able to read the file");
        ExitCode::from(64)
    }
}

fn incorrect_usage() -> ExitCode {
    println!("Usage: lox [script]");
    ExitCode::from(64)
}

fn exit_code_from_error_kind(error_kind: ErrorKind) -> ExitCode {
    match error_kind {
        ErrorKind::Input => ExitCode::from(65),
        ErrorKind::Runtime => ExitCode::from(70),
    }
}
