#[derive(Debug, Clone)]
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
    Lambda(Box<Value>, Box<Value>),
}
