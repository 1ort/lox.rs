use interpreter::Lox;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

mod ast;
mod ast_formatter;
mod interpreter;
mod scanner;

fn main() {
    //     let args: Vec<String> = env::args().collect();
    //     let mut lox = Lox::new();
    //     if args.len() > 2 {
    //         println!("Usage: lox [script]");
    //         process::exit(64);
    //     } else if args.len() == 2 {
    //         run_file(&mut lox, args[1].clone());
    //     } else {
    //         run_prompt(&mut lox);
    //     }

    let expr = ast::Expression::Binary {
        left: Box::new(ast::Expression::Unary {
            operator: ast::UnaryOperator::Minus,
            expression: Box::new(ast::Expression::Literal {
                value: ast::LiteralValue::Number(123.0),
            }),
        }),
        operator: ast::BinaryOperator::Star,
        right: Box::new(ast::Expression::Grouping {
            expression: Box::new(ast::Expression::Literal {
                value: ast::LiteralValue::Number(65.67),
            }),
        }),
    };

    let formatted = ast_formatter::AstFormatter {}.format_expression(&expr);
    println!("{}", formatted)
}

fn run_file(lox: &mut Lox, filename: String) {
    let contents = fs::read_to_string(filename).expect("Should have been able to read the file");
    lox.run(&contents);
    if lox.had_error() {
        process::exit(65);
    }
}

fn run_prompt(lox: &mut Lox) {
    loop {
        let mut line = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line).unwrap();
        if line.trim_end().is_empty() {
            break;
        }
        lox.run(&line);
    }
}
