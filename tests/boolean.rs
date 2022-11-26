use yaql::ast::Visitor;
use yaql::interpreter::Interpreter;
use yaql::lang::Primitive;
use yaql::yaql::ValueParser;

#[test]
fn interpret_and() {
    let interpreter = Interpreter{ context: Primitive::Null };

    let parse_result = ValueParser::new().parse("true and true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("true and false");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("false and false");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("false and true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("true and 12");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Number(num) => assert_eq!(num, 12.0),
        _ => panic!("expected Number"),
    }

    let parse_result = ValueParser::new().parse("null and null");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Null => {},
        _ => panic!("expected Null"),
    }
}

#[test]
fn interpret_or() {
    let interpreter = Interpreter{ context: Primitive::Null };

    let parse_result = ValueParser::new().parse("true or true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("true or false");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("false or false");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("false or true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("12 or true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Number(num) => assert_eq!(num, 12.0),
        _ => panic!("expected Number"),
    }

    let parse_result = ValueParser::new().parse("null or null");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Null => {},
        _ => panic!("expected Null"),
    }
}

#[test]
fn interpret_eq() {
    let interpreter = Interpreter{ context: Primitive::Null };

    let parse_result = ValueParser::new().parse("false = false");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("false != true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("true != false");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("true = true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, true),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("true = false");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("false = true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("false != false");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("true != true");
    let ast = parse_result.unwrap();
    match interpreter.visit(ast).unwrap() {
        Primitive::Boolean(bool) => assert_eq!(bool, false),
        _ => panic!("expected Boolean"),
    }
}
