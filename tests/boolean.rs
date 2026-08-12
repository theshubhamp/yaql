
use yaql::interpreter::Interpreter;
use yaql::lang::Primitive;
use yaql::parser::Parser;

#[test]
fn interpret_and() {
    let mut interpreter = Interpreter::new(Primitive::Null);

    let parse_result = Parser::parse("true and true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("true and false");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("false and false");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("false and true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("true and 12");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Int(num) => assert_eq!(num, 12),
        _ => panic!("expected Number"),
    }

    let parse_result = Parser::parse("null and null");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Null => {},
        _ => panic!("expected Null"),
    }
}

#[test]
fn interpret_or() {
    let mut interpreter = Interpreter::new(Primitive::Null);

    let parse_result = Parser::parse("true or true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("true or false");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("false or false");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("false or true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("12 or true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Int(num) => assert_eq!(num, 12),
        _ => panic!("expected Number"),
    }

    let parse_result = Parser::parse("null or null");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Null => {},
        _ => panic!("expected Null"),
    }
}

#[test]
fn interpret_eq() {
    let mut interpreter = Interpreter::new(Primitive::Null);

    let parse_result = Parser::parse("false = false");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("false != true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("true != false");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("true = true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("true = false");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("false = true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("false != false");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("true != true");
    let ast = parse_result.unwrap();
    match interpreter.visit(&ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }
}
