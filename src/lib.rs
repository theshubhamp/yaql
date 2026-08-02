pub mod ast;
pub mod interpreter;
pub mod json;
pub mod lang;
pub mod lexer;
pub mod parser;

use ast::Visitor;
use interpreter::Interpreter;
use lang::Primitive;

pub enum EvalResult {
    Value(Primitive),
    ParseError(String),
    EvalError(String),
}

pub fn evaluate(expr: &str, context: serde_json::Value) -> EvalResult {
    let context = json::json_to_primitive(&context);
    let ast = match parser::Parser::parse(expr) {
        Ok(ast) => ast,
        Err(e) => return EvalResult::ParseError(format!("{}", e)),
    };
    let interpreter = Interpreter { context };
    match interpreter.visit(ast) {
        Some(value) => EvalResult::Value(value),
        None => EvalResult::EvalError("evaluation returned None".to_string()),
    }
}