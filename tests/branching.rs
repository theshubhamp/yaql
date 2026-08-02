use yaql::ast::Visitor;
use yaql::interpreter::Interpreter;
use yaql::lang::Primitive;
use yaql::parser::Parser;

#[test]
fn interpret_switch() {
    let parse_result = Parser::parse("switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(123) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Int(num) if num == 3));

    let parse_result = Parser::parse("switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(50) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Int(num) if num == 2));

    let parse_result = Parser::parse("switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(-123) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Int(num) if num == 1));
}

#[test]
fn interpret_select_case() {
    let parse_result = Parser::parse("selectCase($ < 10, $ >= 10 and $ < 100)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(123) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Int(num) if num == 2));

    let parse_result = Parser::parse("selectCase($ < 10, $ >= 10 and $ < 100)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(50) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Int(num) if num == 1));

    let parse_result = Parser::parse("selectCase($ < 10, $ >= 10 and $ < 100)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(-123) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Int(num) if num == 0));
}

#[test]
fn interpret_select_all_cases() {
    let parse_result = Parser::parse("selectAllCases($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(1) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 1);
        },
        _ => panic!("expected Array"),
    }

    let parse_result = Parser::parse("selectAllCases($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(7) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 2);
        },
        _ => panic!("expected Array"),
    }

    let parse_result = Parser::parse("selectAllCases($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(12) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 1);
        },
        _ => panic!("expected Array"),
    }
}

#[test]
fn interpret_examine() {
    let parse_result = Parser::parse("examine($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(1) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 2);
        },
        _ => panic!("expected Array"),
    }

    let parse_result = Parser::parse("examine($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(7) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 2);
        },
        _ => panic!("expected Array"),
    }

    let parse_result = Parser::parse("examine($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Int(12) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 2);
        },
        _ => panic!("expected Array"),
    }
}
