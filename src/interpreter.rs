use std::collections::HashMap;
use crate::ast::{Value, Visitor};
use crate::lang::{truthy, BINARY_OPERATORS, FUNCTIONS, Primitive};

pub struct Interpreter {
    pub context: Primitive,
}

impl Visitor<Option<Primitive>> for Interpreter {
    fn visit(&self, value: Value) -> Option<Primitive> {
        return match value {
            Value::StringLiteral(string) => self.visit_string_literal(string),
            Value::NumberLiteral(num) => self.visit_number_literal(num),
            Value::BooleanLiteral(bool) => self.visit_boolean_literal(bool),
            Value::NullLiteral => self.visit_null_literal(),
            Value::Dollar(path) => self.visit_dollar(path),
            Value::FunctionCall(identifier, args, kwargs) => self.visit_function_call(identifier, args, kwargs),
            Value::BinaryOperator(left, op, right) => self.visit_binary_operator(*left, op, *right),
            Value::UnaryOperator(op, operand) => self.visit_unary_operator(op, *operand),
        }
    }

    fn visit_string_literal(&self, string: String) -> Option<Primitive> {
        return Some(Primitive::String(unquote(&string)));
    }

    fn visit_number_literal(&self, num: f64) -> Option<Primitive> {
        return Some(Primitive::Number(num));
    }

    fn visit_boolean_literal(&self, bool: bool) -> Option<Primitive> {
        return Some(Primitive::Boolean(bool));
    }

    fn visit_null_literal(&self) -> Option<Primitive> {
        return Some(Primitive::Null)
    }

    fn visit_dollar(&self, path: String) -> Option<Primitive> {
        if path == "" {
            return Some(self.context.clone())
        }

        todo!()
    }

    fn visit_function_call(&self, identifier: String, args: Vec<Value>, kwargs: Vec<(Value, Value)>) -> Option<Primitive> {
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.visit(arg).unwrap());
        }

        let mut kwarg_values = Vec::new();
        for (kwarg_key, kwarg_value) in kwargs {
            kwarg_values.push((self.visit(kwarg_key).unwrap(), self.visit(kwarg_value).unwrap()));
        }

        return Some(FUNCTIONS.lookup(identifier)(arg_values, kwarg_values));
    }

    fn visit_binary_operator(&self, left: Value, op: String, right: Value) -> Option<Primitive> {
        let left_value = self.visit(left);
        let right_value = self.visit(right);

        if left_value.is_none() || right_value.is_none() {
            return None;
        }

        let left_value = left_value.unwrap();
        let right_value = right_value.unwrap();

        return Some(BINARY_OPERATORS.lookup(op)(left_value, right_value));
    }

    fn visit_unary_operator(&self, op: String, operand: Value) -> Option<Primitive> {
        let val = self.visit(operand)?;
        match op.as_str() {
            "not" => Some(Primitive::Boolean(!truthy(&val))),
            _ => None,
        }
    }
}

fn unquote(raw: &str) -> String {
    if raw.len() < 2 {
        return raw.to_string();
    }
    let quote = raw.chars().next().unwrap();
    let inner = &raw[1..raw.len() - 1];
    if quote == '`' {
        // Verbatim: only \` is escaped, everything else is literal.
        inner.replace("\\`", "`")
    } else if quote == '\'' {
        // Single-quoted: same escapes as JSON strings, plus \'.
        // Convert \' to ' then parse as a JSON string (wrapped in double quotes).
        let json_str = format!("\"{}\"", inner.replace("\\'", "'"));
        serde_json::from_str::<String>(&json_str).unwrap_or_else(|_| inner.to_string())
    } else {
        // Double-quoted: already JSON-compatible.
        let json_str = format!("\"{}\"", inner);
        serde_json::from_str::<String>(&json_str).unwrap_or_else(|_| inner.to_string())
    }
}
