use yaql::ast::Value;
use yaql::yaql::ValueParser;

#[test]
fn parse_numeric() {
    let parse_result = ValueParser::new().parse("1");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::NumberLiteral(_)));
    match ast {
        Value::NumberLiteral(num) => assert!(matches!(num, 1.0)),
        _ => panic!("expected NumberLiteral"),
    }

    let parse_result = ValueParser::new().parse("-1");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::NumberLiteral(_)));
    match ast {
        Value::NumberLiteral(num) => assert!(matches!(num, -1.0)),
        _ => panic!("expected NumberLiteral"),
    }

    let parse_result = ValueParser::new().parse("1.0");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::NumberLiteral(_)));
    match ast {
        Value::NumberLiteral(num) => assert!(matches!(num, 1.0)),
        _ => panic!("expected NumberLiteral"),
    }

    let parse_result = ValueParser::new().parse("-1.1");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::NumberLiteral(_)));
    match ast {
        Value::NumberLiteral(num) => assert!(matches!(num, -1.1)),
        _ => panic!("expected NumberLiteral"),
    }
}

#[test]
fn parse_string() {
    let parse_result = ValueParser::new().parse("'string'");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::StringLiteral(_)));
    match ast {
        Value::StringLiteral(str) => assert_eq!(str, "'string'"),
        _ => panic!("expected StringLiteral"),
    }

    let parse_result = ValueParser::new().parse("\"string\"");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::StringLiteral(_)));
    match ast {
        Value::StringLiteral(str) => assert_eq!(str, "\"string\""),
        _ => panic!("expected StringLiteral"),
    }
}

#[test]
fn parse_boolean() {
    let parse_result = ValueParser::new().parse("true");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::BooleanLiteral(_)));
    match ast {
        Value::BooleanLiteral(bool) => assert!(bool),
        _ => panic!("expected BooleanLiteral"),
    }

    let parse_result = ValueParser::new().parse("false");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::BooleanLiteral(_)));
    match ast {
        Value::BooleanLiteral(bool) => assert!(!bool),
        _ => panic!("expected BooleanLiteral"),
    }
}

#[test]
fn parse_null() {
    let parse_result = ValueParser::new().parse("null");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::NullLiteral));
}

#[test]
fn parse_dollar() {
    let parse_result = ValueParser::new().parse("$context_Value01");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::Dollar(_)));
    match ast {
        Value::Dollar(path) => assert_eq!(path, "context_Value01"),
        _ => panic!("expected Dollar"),
    }
}

#[test]
fn parse_paren() {
    let parse_result = ValueParser::new().parse("($context_Value01)");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::Dollar(_)));
    match ast {
        Value::Dollar(path) => assert_eq!(path, "context_Value01"),
        _ => panic!("expected Dollar"),
    }
}

#[test]
fn parse_function_call() {
    let parse_result = ValueParser::new().parse("abc(1)");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::FunctionCall(_, _, _)));
    match ast {
        Value::FunctionCall(ident, args, kwargs) => {
            assert_eq!(ident, "abc");
            assert!(matches!(args.len(), 1));
        },
        _ => panic!("expected FunctionCall"),
    }

    let parse_result = ValueParser::new().parse("abc(1, true)");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::FunctionCall(_, _, _)));
    match ast {
        Value::FunctionCall(ident, args, kwargs) => {
            assert_eq!(ident, "abc");
            assert!(matches!(args.len(), 2));
            assert!(matches!(kwargs.len(), 0));
        },
        _ => panic!("expected FunctionCall"),
    }

    let parse_result = ValueParser::new().parse("abc(1, true, 'key' => 'value')");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::FunctionCall(_, _, _)));
    match ast {
        Value::FunctionCall(ident, args, kwargs) => {
            assert_eq!(ident, "abc");
            assert!(matches!(args.len(), 2));
            assert!(matches!(kwargs.len(), 1));
        },
        _ => panic!("expected FunctionCall"),
    }

    let parse_result = ValueParser::new().parse("abc(1, true, 'key1' => 'value1', 'key2' => 'value2')");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::FunctionCall(_, _, _)));
    match ast {
        Value::FunctionCall(ident, args, kwargs) => {
            assert_eq!(ident, "abc");
            assert!(matches!(args.len(), 2));
            assert!(matches!(kwargs.len(), 2));
        },
        _ => panic!("expected FunctionCall"),
    }
}

#[test]
fn parse_binary_operators() {
    let parse_result = ValueParser::new().parse("2 + 3");
    let ast = parse_result.unwrap();
    assert!(matches!(ast, Value::BinaryOperator(_, _, _)));
    match ast {
        Value::BinaryOperator(left, op, right) => {
            match *left {
                Value::NumberLiteral(num) => {
                    assert!(matches!(num, 2.0));
                }
                _ => panic!("expected Number"),
            }

            assert_eq!(op, "+");

            match *right {
                Value::NumberLiteral(num) => {
                    assert!(matches!(num, 3.0));
                }
                _ => panic!("expected Number"),
            }
        },
        _ => panic!("expected Binary Operator"),
    }
}
