#[derive(Debug)]
pub enum Value {
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BooleanLiteral(bool),
    NullLiteral,
    Dollar(String),
    FunctionCall(String, Vec<Value>, Vec<(Value, Value)>),
    BinaryOperator(Box<Value>, String, Box<Value>),
    UnaryOperator(String, Box<Value>),
    MethodCall(Box<Value>, bool, String, Vec<Value>, Vec<(Value, Value)>),
    List(Vec<Value>),
    Dict(Vec<(Value, Value)>),
    Index(Box<Value>, Vec<Value>),
}

pub trait Expression<T> {
    fn visit(visitor: dyn Visitor<T>) -> T;
}

pub trait Visitor<T> {
    fn visit(&self, value: Value) -> T;
    fn visit_string_literal(&self, string: String) -> T;
    fn visit_int_literal(&self, num: i64) -> T;
    fn visit_float_literal(&self, num: f64) -> T;
    fn visit_boolean_literal(&self, bool: bool) -> T;
    fn visit_null_literal(&self) -> T;
    fn visit_dollar(&self, path: String) -> T;
    fn visit_function_call(&self, identifier: String, args: Vec<Value>, kwargs: Vec<(Value, Value)>) -> T;
    fn visit_binary_operator(&self, left: Value, op: String, right: Value) -> T;
    fn visit_unary_operator(&self, op: String, operand: Value) -> T;
    fn visit_method_call(&self, receiver: Value, optional: bool, method: String, args: Vec<Value>, kwargs: Vec<(Value, Value)>) -> T;
    fn visit_list(&self, elements: Vec<Value>) -> T;
    fn visit_dict(&self, entries: Vec<(Value, Value)>) -> T;
    fn visit_index(&self, collection: Value, indices: Vec<Value>) -> T;
}
