use yaql::ast::Visitor;
use yaql::interpreter::Interpreter;
use yaql::lang::Primitive;
use yaql::yaql::ValueParser;

#[test]
fn interpret_string() {
    let parse_result = ValueParser::new().parse("'string'");
    let ast = parse_result.unwrap();

    let result = Interpreter{ context: Primitive::Null }.visit(ast).unwrap();
    match result {
        Primitive::String(string) => {
            assert_eq!(string, "'string'");
        }
        _ => panic!("expected String"),
    }

    return;
}

#[test]
fn interpret_number() {
    let parse_result = ValueParser::new().parse("4.2");
    let ast = parse_result.unwrap();

    let result = Interpreter{ context: Primitive::Null }.visit(ast).unwrap();
    match result {
        Primitive::Number(num) => {
            assert_eq!(num, 4.2);
        }
        _ => panic!("expected Number"),
    }

    return;
}

#[test]
fn interpret_boolean() {
    let parse_result = ValueParser::new().parse("false");
    let ast = parse_result.unwrap();

    let result = Interpreter{ context: Primitive::Null }.visit(ast).unwrap();
    match result {
        Primitive::Boolean(bool) => {
            assert_eq!(bool, false);
        }
        _ => panic!("expected Boolean"),
    }

    let parse_result = ValueParser::new().parse("true");
    let ast = parse_result.unwrap();

    let result = Interpreter{ context: Primitive::Null }.visit(ast).unwrap();
    match result {
        Primitive::Boolean(bool) => {
            assert_eq!(bool, true);
        }
        _ => panic!("expected Boolean"),
    }

    return;
}

#[test]
fn interpret_null() {
    let parse_result = ValueParser::new().parse("null");
    let ast = parse_result.unwrap();

    let result = Interpreter{ context: Primitive::Null }.visit(ast).unwrap();
    match result {
        Primitive::Null => {}
        _ => panic!("expected Null"),
    }

    return;
}

#[test]
fn interpret_dollar() {
    let parse_result = ValueParser::new().parse("$");
    let ast = parse_result.unwrap();

    let result = Interpreter{ context: Primitive::Number(12.0) }.visit(ast).unwrap();
    match result {
        Primitive::Number(num) => assert_eq!(num, 12.0),
        _ => panic!("expected Null"),
    }

    return;
}
