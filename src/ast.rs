#[derive(Debug)]
pub enum Value {
    StringLiteral(String),
    NumberLiteral(f64),
    BooleanLiteral(bool),
    NullLiteral,
    Dollar(String),
    FunctionCall(String, Vec<Value>, Vec<(Value, Value)>),
    BinaryOperator(Box<Value>, String, Box<Value>),
    UnaryOperator(String, Box<Value>),
}

pub trait Expression<T> {
    fn visit(visitor: dyn Visitor<T>) -> T;
}

pub trait Visitor<T> {
    fn visit(&self, value: Value) -> T;
    fn visit_string_literal(&self, string: String) -> T;
    fn visit_number_literal(&self, num: f64) -> T;
    fn visit_boolean_literal(&self, bool: bool) -> T;
    fn visit_null_literal(&self) -> T;
    fn visit_dollar(&self, path: String) -> T;
    fn visit_function_call(&self, identifier: String, args: Vec<Value>, kwargs: Vec<(Value, Value)>) -> T;
    fn visit_binary_operator(&self, left: Value, op: String, right: Value) -> T;
}
