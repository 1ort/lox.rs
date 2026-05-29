use runner::Lox;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;
use std::process::exit;

mod ast;
mod class;
mod environment;
mod function;
mod globals;
mod interpreter;
mod interruption;
mod object;
mod parser;
mod resolver;
mod runner;
mod scanner;
mod span;
mod token;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    if args.len() > 2 {
        println!("Usage: lox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        exit(run_file(&mut lox, args[1].clone()))
    } else {
        exit(run_prompt(&mut lox))
    }
}

fn run_file(lox: &mut Lox, filename: String) -> i32 {
    let contents = fs::read_to_string(filename).expect("Should have been able to read the file");
    lox.run(&contents)
}

fn run_prompt(lox: &mut Lox) -> i32 {
    loop {
        let mut line = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line).unwrap();
        if line.trim_end().is_empty() {
            break;
        }
        lox.run(line.trim_end());
    }
    0
}
