use log::{info, error};
use dayum::{
    lexer::scan::Scanner,
    parser::Parser,
    type_checker::TypeChecker,
};
use std::{env, fs};

fn parse_args() -> Result<String, ()> {
    let args: Vec<_> = env::args().collect();
    if args.is_empty() || args.len() != 2 {
        println!("Use: {} [FILE NAME]", args[0]);
        return Err(())
    }

    Ok(args[1].clone())
}

fn run(path: &str) -> Result<(), ()> {
    let src = if path.ends_with(".c") {
        fs::read_to_string(path).map_err(|e| 
            error!("FATAL: cannot access '{}' because {}", path, e)
        )?
    } else {
        error!("FATAL: wrong file format");
        return Err(())
    };

    let mut parser = Parser::new(Scanner::new(&src).peekable());
    let stmts = parser.parse().unwrap();
    let mut type_checker = TypeChecker::new(&stmts);
    type_checker.check().unwrap();
    println!("{:?}", stmts);

    Ok(())
}

fn main() -> Result<(), ()> {
    dayum::logging::init();

    info!("INTERPRETER STARTED!");

    let path = parse_args()?;

    if let Err(_) = run(&path) {
        error!("Process terminated");
    }

    info!("INTERPRETER STOPPED!");
    Ok(())
}
