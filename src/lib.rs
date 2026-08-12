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

impl From<Primitive> for EvalResult {
    fn from(p: Primitive) -> Self {
        EvalResult::Value(p)
    }
}

pub fn evaluate(expr: &str, context: serde_json::Value) -> EvalResult {
    evaluate_with(expr, json::json_to_primitive(&context))
}

/// Evaluate `expr` against a Rust-native `Primitive` context, bypassing JSON
/// (de)serialization. Intended for embedding and for benchmarks that measure
/// dispatch throughput rather than JSON parsing.
pub fn evaluate_with(expr: &str, context: Primitive) -> EvalResult {
    let ast = match parser::Parser::parse(expr) {
        Ok(ast) => ast,
        Err(e) => return EvalResult::ParseError(format!("{}", e)),
    };
    let mut interpreter = Interpreter::new(context);
    match interpreter.visit(ast) {
        Some(value) => {
            // Auto-call top-level lambdas (e.g. `with(5) -> $ + 1`)
            if let Primitive::Lambda(ref lambda) = value {
                // For auto-call: push the last env context (e.g. `with(5)` or `let(...)` result)
                // which is already in the env. Don't push a new context.
                crate::interpreter::eval_lambda_auto(lambda).into()
            } else {
                EvalResult::Value(value)
            }
        }
        None => EvalResult::EvalError("evaluation returned None".to_string()),
    }
}