
use yaql::interpreter::Interpreter;
use yaql::lang::Primitive;
use yaql::parser::Parser;

#[test]
fn interpret_string() {
    let parse_result = Parser::parse("'string'");
    let ast = parse_result.unwrap();

    let result = Interpreter::new(Primitive::Null).visit(&ast).unwrap();
    match result {
        Primitive::String(string) => {
            assert_eq!(string, "string");
        }
        _ => panic!("expected String"),
    }

}

#[test]
fn interpret_number() {
    let parse_result = Parser::parse("4.2");
    let ast = parse_result.unwrap();

    let result = Interpreter::new(Primitive::Null).visit(&ast).unwrap();
    match result {
        Primitive::Float(num) => {
            assert_eq!(num, 4.2);
        }
        _ => panic!("expected Float"),
    }

}

#[test]
fn interpret_boolean() {
    let parse_result = Parser::parse("false");
    let ast = parse_result.unwrap();

    let result = Interpreter::new(Primitive::Null).visit(&ast).unwrap();
    match result {
        Primitive::Boolean(bool) => {
            assert!(!bool);
        }
        _ => panic!("expected Boolean"),
    }

    let parse_result = Parser::parse("true");
    let ast = parse_result.unwrap();

    let result = Interpreter::new(Primitive::Null).visit(&ast).unwrap();
    match result {
        Primitive::Boolean(bool) => {
            assert!(bool);
        }
        _ => panic!("expected Boolean"),
    }

}

#[test]
fn interpret_null() {
    let parse_result = Parser::parse("null");
    let ast = parse_result.unwrap();

    let result = Interpreter::new(Primitive::Null).visit(&ast).unwrap();
    match result {
        Primitive::Null => {}
        _ => panic!("expected Null"),
    }

}

#[test]
fn interpret_dollar() {
    let parse_result = Parser::parse("$");
    let ast = parse_result.unwrap();

    let result = Interpreter::new(Primitive::Int(12)).visit(&ast).unwrap();
    match result {
        Primitive::Int(num) => assert_eq!(num, 12),
        _ => panic!("expected Null"),
    }

}
