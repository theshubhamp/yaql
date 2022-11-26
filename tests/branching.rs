use yaql::ast::Visitor;
use yaql::interpreter::Interpreter;
use yaql::lang::Primitive;
use yaql::yaql::ValueParser;

#[test]
fn interpret_switch() {
    let parse_result = ValueParser::new().parse("switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(123.0) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Number(num) if num == 3.0));

    let parse_result = ValueParser::new().parse("switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(50.0) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Number(num) if num == 2.0));

    let parse_result = ValueParser::new().parse("switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(-123.0) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Number(num) if num == 1.0));
}

#[test]
fn interpret_select_case() {
    let parse_result = ValueParser::new().parse("selectCase($ < 10, $ >= 10 and $ < 100)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(123.0) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Number(num) if num == 2.0));

    let parse_result = ValueParser::new().parse("selectCase($ < 10, $ >= 10 and $ < 100)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(50.0) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Number(num) if num == 1.0));

    let parse_result = ValueParser::new().parse("selectCase($ < 10, $ >= 10 and $ < 100)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(-123.0) };
    let result = interpreter.visit(ast).unwrap();
    assert!(matches!(result, Primitive::Number(num) if num == 0.0));
}

#[test]
fn interpret_select_all_cases() {
    let parse_result = ValueParser::new().parse("selectAllCases($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(1.0) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 1);
        },
        _ => panic!("expected Array"),
    }

    let parse_result = ValueParser::new().parse("selectAllCases($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(7.0) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 2);
        },
        _ => panic!("expected Array"),
    }

    let parse_result = ValueParser::new().parse("selectAllCases($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(12.0) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 1);
        },
        _ => panic!("expected Array"),
    }
}

#[test]
fn interpret_examine() {
    let parse_result = ValueParser::new().parse("examine($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(1.0) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 2);
        },
        _ => panic!("expected Array"),
    }

    let parse_result = ValueParser::new().parse("examine($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(7.0) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 2);
        },
        _ => panic!("expected Array"),
    }

    let parse_result = ValueParser::new().parse("examine($ < 10, $ > 5)");
    let ast = parse_result.unwrap();
    let interpreter = Interpreter { context: Primitive::Number(12.0) };
    match interpreter.visit(ast).unwrap() {
        Primitive::Array(array) => {
            assert_eq!(array.len(), 2);
        },
        _ => panic!("expected Array"),
    }
}
