use crate::{
    compile::{parser::parse_program, resolver::resolve_program, scanner::scan_tokens},
    error::{ariadne::AriadneReporter, reporter::ErrorReporter},
    runtime::interpreter::Interpreter,
};
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
mod runtime;
mod span;

enum ErrorKind {
    Input,
    Runtime,
}

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
        let _ = run(&mut lox, line.trim_end(), "repl");
    }
    ExitCode::from(0)
}

fn run_file(filename: &OsStr) -> ExitCode {
    let contents = fs::read_to_string(filename);
    if let Ok(contents) = contents {
        let mut lox = Lox::new();
        match run(&mut lox, &contents, filename.to_str().unwrap()) {
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

struct Lox {
    interpreter: Interpreter,
}

fn run(lox: &mut Lox, source: &str, source_name: &str) -> Result<(), ErrorKind> {
    let error_reporter = AriadneReporter::new(source, source_name);

    let tokens = scan_tokens(source);
    let mut program = parse_program(&tokens, &error_reporter).map_err(|_| ErrorKind::Input)?;
    resolve_program(&mut program, &error_reporter).map_err(|_| ErrorKind::Input)?;

    if let Err(err) = lox.interpreter.exec(&program) {
        error_reporter.report(&err);
        Err(ErrorKind::Runtime)
    } else {
        Ok(())
    }
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            interpreter: Interpreter::new(),
        }
    }
}
