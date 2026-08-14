use crate::ast::Value;
use crate::lang::{truthy, Primitive};
use crate::lang::functions::{EvalError, FUNCTIONS, dispatch};
use crate::lang::primitive::LambdaBody;
use std::sync::Arc;

pub use crate::lang::primitive::LambdaBody as Lambda;

// --- Interpreter ---

pub struct Interpreter {
    pub contexts: Vec<Primitive>,
    pub current_func: Option<String>,
}

impl Interpreter {
    pub fn new(context: Primitive) -> Self {
        Interpreter { contexts: vec![context], current_func: None }
    }

    pub fn context(&self) -> &Primitive {
        self.contexts.last().expect("context stack is empty")
    }

    fn dollar_lookup(&self, path: &str) -> Option<Primitive> {
        let ctx = self.context();
        if path.is_empty() {
            return Some(ctx.clone());
        }
        // Numeric: $1, $2 — positional lambda args
        if path.chars().all(|c| c.is_ascii_digit()) && !path.is_empty() {
            let n: usize = path.parse().unwrap_or(0);
            if n == 0 { return Some(Primitive::Null); }
            // Walk stack for an Array context (2-arg lambda args [acc, element])
            for ctx in self.contexts.iter().rev() {
                if let Primitive::Array(arr) = ctx {
                    if arr.len() >= 2 {
                        return arr.get(n - 1).cloned();
                    }
                }
            }
            // Single-arg lambda: $1 = context
            if n == 1 { return Some(self.context().clone()); }
            return Some(Primitive::Null);
        }
        // Named: $a, $foo — walk context stack for a Map with that key
        for ctx in self.contexts.iter().rev() {
            if let Primitive::Map(map) = ctx {
                if let Some(v) = map.get(path) {
                    return Some(v.clone());
                }
            }
        }
        Some(Primitive::Null)
    }

    pub fn push_context(&mut self, val: Primitive) {
        self.contexts.push(val);
    }

    pub fn pop_context(&mut self) {
        self.contexts.pop();
    }
}

/// Free function to evaluate a lambda with a given argument.
/// Used by functions that receive Primitive::Lambda as an argument.
pub fn eval_lambda(lambda: &LambdaBody, arg: Primitive) -> Result<Primitive, EvalError> {
    let mut interp = Interpreter { contexts: (*lambda.env).clone(), current_func: None };
    interp.push_context(arg);
    eval_body(&mut interp, &lambda.body)
}

/// Auto-call a lambda at top level — don't push a new context, just use the env.
pub fn eval_lambda_auto(lambda: &LambdaBody) -> Result<Primitive, EvalError> {
    let mut interp = Interpreter { contexts: (*lambda.env).clone(), current_func: None };
    eval_body(&mut interp, &lambda.body)
}

pub fn eval_body(interp: &mut Interpreter, body: &Value) -> Result<Primitive, EvalError> {
    match body {
        Value::Lambda(left, right) => {
            let env_val = interp.visit(left)?;
            interp.push_context(env_val);
            eval_body(interp, right)
        }
        other => interp.visit(other),
    }
}

impl Interpreter {
    /// Evaluate a `Value` AST node. Takes `&Value` so the lambda hot path can
    /// evaluate a body repeatedly without cloning the AST on every element.
    pub fn visit(&mut self, value: &Value) -> Result<Primitive, EvalError> {
        match value {
            Value::StringLiteral(string) => Ok(Primitive::String(unquote(string))),
            Value::IntLiteral(num) => Ok(Primitive::Int(*num)),
            Value::FloatLiteral(num) => Ok(Primitive::Float(*num)),
            Value::BooleanLiteral(b) => Ok(Primitive::Boolean(*b)),
            Value::NullLiteral => Ok(Primitive::Null),
            Value::Dollar(path) => {
                Ok(self.dollar_lookup(path).unwrap_or(Primitive::Null))
            }
            Value::Lambda(left, right) => {
                Ok(Primitive::Lambda(LambdaBody {
                    body: Box::new(Value::Lambda(left.clone(), right.clone())),
                    env: Arc::new(self.contexts.clone()),
                }))
            }
            Value::FunctionCall(identifier, args, kwargs) => {
                self.current_func = Some(identifier.clone());
                let overloads = FUNCTIONS.lookup(identifier);
                let arg_values = self.eval_args(args, true)?;
                let kwarg_values = self.eval_kwargs(kwargs)?;
                self.current_func = None;
                dispatch(overloads, arg_values, kwarg_values)
            }
            Value::BinaryOperator(left, op, right) => {
                match op.as_str() {
                    "and" => {
                        let left_value = self.visit(left)?;
                        if !truthy(&left_value) { return Ok(left_value); }
                        self.visit(right)
                    }
                    "or" => {
                        let left_value = self.visit(left)?;
                        if truthy(&left_value) { return Ok(left_value); }
                        self.visit(right)
                    }
                    _ => {
                        let left_value = self.visit(left)?;
                        let right_value = self.visit(right)?;
                        dispatch(FUNCTIONS.lookup(op), vec![left_value, right_value], vec![])
                    }
                }
            }
            Value::UnaryOperator(op, operand) => {
                let val = self.visit(operand)?;
                match op.as_str() {
                    "not" => Ok(Primitive::Boolean(!truthy(&val))),
                    "-" => match val {
                        Primitive::Int(n) => Ok(Primitive::Int(-n)),
                        Primitive::Float(n) => Ok(Primitive::Float(-n)),
                        _ => Ok(Primitive::Null),
                    },
                    "+" => Ok(val),
                    _ => Ok(Primitive::Null),
                }
            }
            Value::MethodCall(receiver, optional, method, args, kwargs) => {
                let receiver_value = self.visit(receiver)?;
                if *optional && matches!(receiver_value, Primitive::Null) {
                    return Ok(Primitive::Null);
                }
                self.current_func = Some(method.clone());
                let overloads = FUNCTIONS.lookup(method);
                let mut arg_values = vec![receiver_value];
                arg_values.extend(self.eval_args(args, false)?);
                let kwarg_values = self.eval_kwargs(kwargs)?;
                self.current_func = None;
                dispatch(overloads, arg_values, kwarg_values)
            }
            Value::List(elements) => {
                let mut items = Vec::new();
                for e in elements {
                    items.push(self.visit(e)?);
                }
                Ok(Primitive::Array(items))
            }
            Value::Dict(entries) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in entries {
                    let key = match self.visit(k)? {
                        Primitive::String(s) => s,
                        Primitive::Int(n) => n.to_string(),
                        Primitive::Boolean(b) => b.to_string(),
                        Primitive::Null => "null".to_string(),
                        _ => continue,
                    };
                    map.insert(key, self.visit(v)?);
                }
                Ok(Primitive::Map(map))
            }
            Value::Index(collection, indices) => {
                let coll = self.visit(collection)?;
                let mut idx_values: Vec<Primitive> = Vec::new();
                for v in indices {
                    if let Value::BinaryOperator(left, op, right) = v {
                        if op == "=>" {
                            let key = self.visit(left)?;
                            let val = self.visit(right)?;
                            if let Primitive::String(k) = &key {
                                if let Primitive::Map(map) = &coll {
                                    if let Some(v) = map.get(k) {
                                        return Ok(v.clone());
                                    }
                                }
                            }
                            return Ok(val);
                        }
                        idx_values.push(self.visit(v)?);
                    } else {
                        idx_values.push(self.visit(v)?);
                    }
                }
                let idx = match idx_values.first() {
                    Some(i) => i.clone(),
                    None => Primitive::Null,
                };
                let default = idx_values.get(1).cloned();
                let result = match (&coll, &idx) {
                    (Primitive::Array(arr), Primitive::Int(i)) => {
                        let len = arr.len() as i64;
                        let pos = if *i < 0 { *i + len } else { *i };
                        arr.get(pos as usize).cloned().or(default)
                    }
                    (Primitive::Set(arr), Primitive::Int(i)) => {
                        let len = arr.len() as i64;
                        let pos = if *i < 0 { *i + len } else { *i };
                        arr.get(pos as usize).cloned().or(default)
                    }
                    (Primitive::Map(map), Primitive::String(key)) => map.get(key).cloned().or(default),
                    (Primitive::Map(map), Primitive::Int(i)) => map.get(&i.to_string()).cloned().or(default),
                    _ => default,
                };
                Ok(result.unwrap_or(Primitive::Null))
            }
        }
    }

    fn eval_args(&mut self, args: &[Value], first_is_collection: bool) -> Result<Vec<Primitive>, EvalError> {
        let mut out = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            let is_lambda_pos = !first_is_collection || i > 0;
            out.push(self.eval_arg(a, is_lambda_pos)?);
        }
        Ok(out)
    }

    fn eval_kwargs(&mut self, kwargs: &[(Value, Value)]) -> Result<Vec<(Primitive, Primitive)>, EvalError> {
        let mut out = Vec::with_capacity(kwargs.len());
        for (k, v) in kwargs {
            let k = self.visit(k)?;
            let v = self.eval_arg(v, true)?;
            out.push((k, v));
        }
        Ok(out)
    }

    fn eval_arg(&mut self, arg: &Value, is_lambda_pos: bool) -> Result<Primitive, EvalError> {
        Ok(match arg {
            Value::Lambda(left, right) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(Value::Lambda(left.clone(), right.clone())),
                    env: Arc::new(self.contexts.clone()),
                })
            }
            Value::Dollar(_) if is_lambda_pos && is_lambda_context(&self.current_func) && !is_strict_lambda(&self.current_func) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(arg.clone()),
                    env: Arc::new(self.contexts.clone()),
                })
            }
            Value::Dollar(_) => self.visit(arg)?,
            other if is_lambda_pos && is_lambda_context(&self.current_func)
                && !is_strict_lambda(&self.current_func)
                && contains_dollar(other) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(other.clone()),
                    env: Arc::new(self.contexts.clone()),
                })
            }
            other if is_lambda_pos && is_strict_lambda(&self.current_func)
                && contains_numeric_dollar(other) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(other.clone()),
                    env: Arc::new(self.contexts.clone()),
                })
            }
            other => self.visit(other)?,
        })
    }

    pub fn eval_lambda(&mut self, lambda: &LambdaBody, arg: Primitive) -> Result<Primitive, EvalError> {
        let saved = std::mem::take(&mut self.contexts);
        self.contexts = (*lambda.env).clone();
        self.push_context(arg);
        let result = self.visit(&lambda.body);
        self.contexts = saved;
        result
    }
}

fn unquote(raw: &str) -> String {
    if raw.len() < 2 {
        return raw.to_string();
    }
    let first = raw.chars().next().unwrap();
    let last = raw.chars().last().unwrap();
    if !((first == '\'' || first == '"' || first == '`') && first == last) {
        return raw.to_string();
    }
    let inner = &raw[1..raw.len() - 1];
    if first == '`' {
        inner.replace("\\`", "`")
    } else if first == '\'' {
        let json_str = format!("\"{}\"", inner.replace("\\'", "'"));
        serde_json::from_str::<String>(&json_str).unwrap_or_else(|_| inner.to_string())
    } else {
        let json_str = format!("\"{}\"", inner);
        serde_json::from_str::<String>(&json_str).unwrap_or_else(|_| inner.to_string())
    }
}

/// Check if a Value AST contains a bare `$` (Dollar with empty path),
/// indicating it should be treated as an implicit lambda.
fn contains_dollar(v: &Value) -> bool {
    match v {
        Value::Dollar(path) => path.is_empty() || path.chars().all(|c| c.is_ascii_digit()),
        Value::BinaryOperator(l, _, r) => contains_dollar(l) || contains_dollar(r),
        Value::UnaryOperator(_, e) => contains_dollar(e),
        Value::MethodCall(r, _, _, args, kwargs) => {
            contains_dollar(r) || args.iter().any(contains_dollar)
                || kwargs.iter().any(|(k, v)| contains_dollar(k) || contains_dollar(v))
        }
        Value::FunctionCall(_, args, kwargs) => {
            args.iter().any(contains_dollar)
                || kwargs.iter().any(|(k, v)| contains_dollar(k) || contains_dollar(v))
        }
        Value::List(elements) => elements.iter().any(contains_dollar),
        Value::Dict(entries) => entries.iter().any(|(k, v)| contains_dollar(k) || contains_dollar(v)),
        Value::Index(coll, indices) => contains_dollar(coll) || indices.iter().any(contains_dollar),
        Value::Lambda(l, r) => contains_dollar(l) || contains_dollar(r),
        _ => false,
    }
}

/// Check if a Value contains $N (numeric dollar) — for strict lambda functions
fn contains_numeric_dollar(v: &Value) -> bool {
    match v {
        Value::Dollar(path) => !path.is_empty() && path.chars().all(|c| c.is_ascii_digit()),
        Value::BinaryOperator(l, _, r) => contains_numeric_dollar(l) || contains_numeric_dollar(r),
        Value::UnaryOperator(_, e) => contains_numeric_dollar(e),
        Value::MethodCall(r, _, _, args, kwargs) => {
            contains_numeric_dollar(r) || args.iter().any(contains_numeric_dollar)
                || kwargs.iter().any(|(k, v)| contains_numeric_dollar(k) || contains_numeric_dollar(v))
        }
        Value::FunctionCall(_, args, kwargs) => {
            args.iter().any(contains_numeric_dollar)
                || kwargs.iter().any(|(k, v)| contains_numeric_dollar(k) || contains_numeric_dollar(v))
        }
        Value::List(elements) => elements.iter().any(contains_numeric_dollar),
        Value::Dict(entries) => entries.iter().any(|(k, v)| contains_numeric_dollar(k) || contains_numeric_dollar(v)),
        Value::Index(coll, indices) => contains_numeric_dollar(coll) || indices.iter().any(contains_numeric_dollar),
        Value::Lambda(l, r) => contains_numeric_dollar(l) || contains_numeric_dollar(r),
        _ => false,
    }
}

/// Functions where bare $ should NOT be treated as identity lambda
/// (they have non-lambda array args that happen to be $)
fn is_strict_lambda(func: &Option<String>) -> bool {
    matches!(func.as_deref(), Some("join") | Some("mergeWith"))
}

/// Functions that treat their arguments as implicit lambdas when they contain `$`.
fn is_lambda_context(func: &Option<String>) -> bool {
    match func.as_deref() {
        Some("where") | Some("select") | Some("selectMany")
        | Some("orderBy") | Some("orderByDescending")
        | Some("thenBy") | Some("thenByDescending")
        | Some("takeWhile") | Some("skipWhile")
        | Some("any") | Some("all")
        | Some("distinct") | Some("indexWhere") | Some("lastIndexWhere")
        | Some("aggregate") | Some("reduce") | Some("accumulate")
        | Some("groupBy") | Some("toDict") | Some("memorize")
        | Some("splitWhere") | Some("sliceWhere")
        | Some("join") | Some("mergeWith")
        | Some("search") | Some("searchAll") | Some("replaceBy")
        | Some("generate") | Some("generateMany")
        => true,
        _ => false,
    }
}
