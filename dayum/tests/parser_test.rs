use dayum::parser::Parser;
use dayum::lexer::scan::Scanner;

const PREFIX: &str = "./tests/parser_cases/";

const FILE_CASES: &[&str] = &[
    "function_definition.c",
    "global_variable.c",
    "if-statement.c",
    "for-statement.c"
];

#[test]
fn it_works() {
    for (idx, file) in FILE_CASES.iter().enumerate() {
        let source = std::fs::read_to_string(format!("{}{}", PREFIX, file).as_str()).expect("Trouble reading file!");
        let scanner = Scanner::new(source.as_str());
        let mut parser = Parser::new(scanner.peekable());
        match parser.parse() {
            Ok(_) => {
                println!("Test {} succeeded in file {}", idx + 1, file)
            },
            Err(err) => {
                panic!("Test {} failed with error: {}, in file {}", idx + 1, err, file)
            }
        }
    }
}
