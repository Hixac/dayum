use dayum::parser::ast::TopLevelStmt;
use dayum::type_checker::TypeChecker;
use dayum::parser::Parser;
use dayum::lexer::scan::Scanner;

const PREFIX: &str = "./tests/type_checker_cases/";

const FILE_CASES: &[&str] = &[
    "function_definition.c",
    "global_variable.c",
    "if-statement.c",
    "for-statement.c"
];

fn type_check(stmts: Vec<TopLevelStmt>) -> () {
    let mut checker = TypeChecker::new();
    if checker.check(&stmts).is_err() {

    }
}

#[test]
fn it_works() {
    for (idx, file) in FILE_CASES.iter().enumerate() {
        let source = std::fs::read_to_string(format!("{}{}", PREFIX, file).as_str()).expect("Trouble reading file!");
        let scanner = Scanner::new(source.as_str());
        let mut parser = Parser::new(scanner.peekable());
        match parser.parse() {
            Ok(stmts) => {
                type_check(stmts);
            },
            Err(err) => {
                panic!("Test {} failed with error: {}, in file {}", idx + 1, err, file)
            }
        }
    }
}



/*
void function(float w, char *r, int) {
    int hello[3];
    hello[1] = 2.0;
    {
        {
            float lol;
        }
        lol;
    }
    int hello;
    return 1;
}

void function() {

}

void other_fuckntion() {
    function(1, 2, 3);
}
*/
