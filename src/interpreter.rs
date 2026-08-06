use crate::ast::{Value, Visitor};
use crate::lang::{truthy, ArgSpec, BINARY_OPERATORS, FUNCTIONS, Primitive, Spec};
use crate::lang::primitive::LambdaBody;

pub use crate::lang::primitive::LambdaBody as Lambda;

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

    fn eval_arg(&mut self, arg: Value, is_lambda_pos: bool) -> Primitive {
        match arg {
            Value::Lambda(left, right) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(Value::Lambda(left, right)),
                    env: self.contexts.clone(),
                })
            }
            // Bare $ in a lambda-context function → identity lambda (except for strict funcs)
            Value::Dollar(_) if is_lambda_pos && is_lambda_context(&self.current_func) && !is_strict_lambda(&self.current_func) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(arg),
                    env: self.contexts.clone(),
                })
            }
            // Bare $ in non-lambda-context → evaluate normally
            Value::Dollar(_) => self.visit_mut(arg).unwrap_or(Primitive::Null),
            other if is_lambda_pos && is_lambda_context(&self.current_func)
                && !is_strict_lambda(&self.current_func)
                && contains_dollar(&other) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(other),
                    env: self.contexts.clone(),
                })
            }
            // For strict lambda functions (join, mergeWith): only wrap if contains $N (numeric)
            other if is_lambda_pos && is_strict_lambda(&self.current_func)
                && contains_numeric_dollar(&other) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(other),
                    env: self.contexts.clone(),
                })
            }
            other => self.visit_mut(other).unwrap_or(Primitive::Null),
        }
    }

    fn eval_args(&mut self, args: Vec<Value>, first_is_collection: bool) -> Vec<Primitive> {
        args.into_iter().enumerate().map(|(i, a)| {
            let is_lambda_pos = !first_is_collection || i > 0;
            self.eval_arg(a, is_lambda_pos)
        }).collect()
    }

    fn eval_kwargs(&mut self, kwargs: Vec<(Value, Value)>) -> Vec<(Primitive, Primitive)> {
        kwargs.into_iter()
            .filter_map(|(k, v)| {
                let k = self.visit_mut(k).unwrap_or(Primitive::Null);
                let v = self.eval_arg(v, true);
                Some((k, v))
            })
            .collect()
    }

    pub fn eval_lambda(&mut self, lambda: &LambdaBody, arg: Primitive) -> Primitive {
        let saved = std::mem::take(&mut self.contexts);
        self.contexts = lambda.env.clone();
        self.push_context(arg);
        let result = self.visit_mut((*lambda.body).clone()).unwrap_or(Primitive::Null);
        self.contexts = saved;
        result
    }
}

/// Free function to evaluate a lambda with a given argument.
/// Used by functions that receive Primitive::Lambda as an argument.
pub fn eval_lambda(lambda: &LambdaBody, arg: Primitive) -> Primitive {
    let mut interp = Interpreter { contexts: lambda.env.clone(), current_func: None };
    interp.push_context(arg);
    eval_body(&mut interp, &lambda.body)
}

/// Auto-call a lambda at top level — don't push a new context, just use the env.
pub fn eval_lambda_auto(lambda: &LambdaBody) -> Primitive {
    let mut interp = Interpreter { contexts: lambda.env.clone(), current_func: None };
    eval_body(&mut interp, &lambda.body)
}

pub fn eval_body(interp: &mut Interpreter, body: &Value) -> Primitive {
    match body {
        Value::Lambda(left, right) => {
            let env_val = interp.visit_mut((**left).clone()).unwrap_or(Primitive::Null);
            interp.push_context(env_val);
            eval_body(interp, right)
        }
        other => interp.visit_mut(other.clone()).unwrap_or(Primitive::Null),
    }
}

impl Visitor<Option<Primitive>> for Interpreter {
    fn visit(&self, value: Value) -> Option<Primitive> {
        let mut this = Interpreter { contexts: self.contexts.clone(), current_func: self.current_func.clone() };
        this.visit_mut(value)
    }

    fn visit_string_literal(&self, string: String) -> Option<Primitive> {
        Some(Primitive::String(unquote(&string)))
    }

    fn visit_int_literal(&self, num: i64) -> Option<Primitive> {
        Some(Primitive::Int(num))
    }

    fn visit_float_literal(&self, num: f64) -> Option<Primitive> {
        Some(Primitive::Float(num))
    }

    fn visit_boolean_literal(&self, bool: bool) -> Option<Primitive> {
        Some(Primitive::Boolean(bool))
    }

    fn visit_null_literal(&self) -> Option<Primitive> {
        Some(Primitive::Null)
    }

    fn visit_dollar(&self, path: String) -> Option<Primitive> {
        self.dollar_lookup(&path)
    }

    fn visit_function_call(&self, identifier: String, args: Vec<Value>, kwargs: Vec<(Value, Value)>) -> Option<Primitive> {
        self.visit(Value::FunctionCall(identifier, args, kwargs))
    }

    fn visit_binary_operator(&self, left: Value, op: String, right: Value) -> Option<Primitive> {
        match op.as_str() {
            "and" => {
                let left_value = self.visit(left.clone())?;
                if !truthy(&left_value) { return Some(left_value); }
                self.visit(right)
            }
            "or" => {
                let left_value = self.visit(left.clone())?;
                if truthy(&left_value) { return Some(left_value); }
                self.visit(right)
            }
            _ => {
                let left_value = self.visit(left)?;
                let right_value = self.visit(right)?;
                Some(BINARY_OPERATORS.lookup(op)(left_value, right_value))
            }
        }
    }

    fn visit_unary_operator(&self, op: String, operand: Value) -> Option<Primitive> {
        let val = self.visit(operand)?;
        match op.as_str() {
            "not" => Some(Primitive::Boolean(!truthy(&val))),
            "-" => match val {
                Primitive::Int(n) => Some(Primitive::Int(-n)),
                Primitive::Float(n) => Some(Primitive::Float(-n)),
                _ => None,
            },
            "+" => Some(val),
            _ => None,
        }
    }

    fn visit_method_call(&self, receiver: Value, optional: bool, method: String, args: Vec<Value>, kwargs: Vec<(Value, Value)>) -> Option<Primitive> {
        self.visit(Value::MethodCall(Box::new(receiver), optional, method, args, kwargs))
    }

    fn visit_list(&self, elements: Vec<Value>) -> Option<Primitive> {
        let mut items = Vec::new();
        for e in elements {
            items.push(self.visit(e)?);
        }
        Some(Primitive::Array(items))
    }

    fn visit_dict(&self, entries: Vec<(Value, Value)>) -> Option<Primitive> {
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
        Some(Primitive::Map(map))
    }

    fn visit_index(&self, collection: Value, indices: Vec<Value>) -> Option<Primitive> {
        let coll = self.visit(collection)?;
        let mut idx_values: Vec<Primitive> = Vec::new();
        for v in indices {
            if let Value::BinaryOperator(left, op, right) = v {
                if op == "=>" {
                    let key = self.visit(*left)?;
                    let val = self.visit(*right)?;
                    if let Primitive::String(k) = &key {
                        if let Primitive::Map(map) = &coll {
                            if let Some(v) = map.get(k) {
                                return Some(v.clone());
                            }
                        }
                    }
                    return Some(val);
                }
                idx_values.push(self.visit(Value::BinaryOperator(left, op, right))?);
            } else {
                idx_values.push(self.visit(v)?);
            }
        }
        let idx = idx_values.first()?.clone();
        let default = idx_values.get(1).cloned();
        match (&coll, &idx) {
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
        }
    }

    fn visit_lambda(&self, left: Value, right: Value) -> Option<Primitive> {
        let mut this = Interpreter { contexts: self.contexts.clone(), current_func: None };
        let env_val = this.visit(left)?;
        this.push_context(env_val);
        let result = this.visit(right)?;
        // Auto-call if body result is not a lambda — i.e., evaluate immediately
        // with env as context
        Some(result)
    }
}

impl Interpreter {
    pub fn visit_mut(&mut self, value: Value) -> Option<Primitive> {
        match value {
            Value::StringLiteral(string) => Some(Primitive::String(unquote(&string))),
            Value::IntLiteral(num) => Some(Primitive::Int(num)),
            Value::FloatLiteral(num) => Some(Primitive::Float(num)),
            Value::BooleanLiteral(b) => Some(Primitive::Boolean(b)),
            Value::NullLiteral => Some(Primitive::Null),
            Value::Dollar(path) => {
                self.dollar_lookup(&path)
            }
            Value::Lambda(left, right) => {
                Primitive::Lambda(LambdaBody {
                    body: Box::new(Value::Lambda(left, right)),
                    env: self.contexts.clone(),
                }).into()
            }
            Value::FunctionCall(identifier, args, kwargs) => {
                self.current_func = Some(identifier.clone());
                let overloads = FUNCTIONS.lookup(identifier);
                let arg_values = self.eval_args(args, true);
                let kwarg_values = self.eval_kwargs(kwargs);
                self.current_func = None;
                Some(dispatch(overloads, arg_values, kwarg_values))
            }
            Value::BinaryOperator(left, op, right) => {
                match op.as_str() {
                    "and" => {
                        let left_value = self.visit_mut(*left)?;
                        if !truthy(&left_value) { return Some(left_value); }
                        self.visit_mut(*right)
                    }
                    "or" => {
                        let left_value = self.visit_mut(*left)?;
                        if truthy(&left_value) { return Some(left_value); }
                        self.visit_mut(*right)
                    }
                    _ => {
                        let left_value = self.visit_mut(*left)?;
                        let right_value = self.visit_mut(*right)?;
                        Some(BINARY_OPERATORS.lookup(op)(left_value, right_value))
                    }
                }
            }
            Value::UnaryOperator(op, operand) => {
                let val = self.visit_mut(*operand)?;
                match op.as_str() {
                    "not" => Some(Primitive::Boolean(!truthy(&val))),
                    "-" => match val {
                        Primitive::Int(n) => Some(Primitive::Int(-n)),
                        Primitive::Float(n) => Some(Primitive::Float(-n)),
                        _ => None,
                    },
                    "+" => Some(val),
                    _ => None,
                }
            }
            Value::MethodCall(receiver, optional, method, args, kwargs) => {
                let receiver_value = self.visit_mut(*receiver)?;
                if optional && matches!(receiver_value, Primitive::Null) {
                    return Some(Primitive::Null);
                }
                self.current_func = Some(method.clone());
                let overloads = FUNCTIONS.lookup(method);
                let mut arg_values = vec![receiver_value];
                arg_values.extend(self.eval_args(args, false));
                let kwarg_values = self.eval_kwargs(kwargs);
                self.current_func = None;
                Some(dispatch(overloads, arg_values, kwarg_values))
            }
            Value::List(elements) => {
                let mut items = Vec::new();
                for e in elements {
                    items.push(self.visit_mut(e)?);
                }
                Some(Primitive::Array(items))
            }
            Value::Dict(entries) => {
                let mut map = std::collections::HashMap::new();
                for (k, v) in entries {
                    let key = match self.visit_mut(k)? {
                        Primitive::String(s) => s,
                        Primitive::Int(n) => n.to_string(),
                        Primitive::Boolean(b) => b.to_string(),
                        Primitive::Null => "null".to_string(),
                        _ => continue,
                    };
                    map.insert(key, self.visit_mut(v)?);
                }
                Some(Primitive::Map(map))
            }
            Value::Index(collection, indices) => {
                let coll = self.visit_mut(*collection)?;
                let mut idx_values: Vec<Primitive> = Vec::new();
                for v in indices {
                    if let Value::BinaryOperator(left, op, right) = v {
                        if op == "=>" {
                            let key = self.visit_mut(*left)?;
                            let val = self.visit_mut(*right)?;
                            if let Primitive::String(k) = &key {
                                if let Primitive::Map(map) = &coll {
                                    if let Some(v) = map.get(k) {
                                        return Some(v.clone());
                                    }
                                }
                            }
                            return Some(val);
                        }
                        idx_values.push(self.visit_mut(Value::BinaryOperator(left, op, right))?);
                    } else {
                        idx_values.push(self.visit_mut(v)?);
                    }
                }
                let idx = idx_values.first()?.clone();
                let default = idx_values.get(1).cloned();
                match (&coll, &idx) {
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
                }
            }
        }
    }
}

/// Find the best matching overload and call it.
fn dispatch(overloads: Vec<Spec>, args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut overloads = overloads;
    overloads.sort_by(|a, b| {
        let score = |s: &Spec| {
            s.arg_types.iter().map(|ty| match ty {
                crate::lang::Type::Any => 3,
                crate::lang::Type::Number => 2,
                _ => 0,
            }).sum::<u32>()
        };
        let sa = score(a);
        let sb = score(b);
        sa.cmp(&sb).then_with(|| {
            b.arg_types.len().cmp(&a.arg_types.len())
        })
    });
    let typed: Vec<&Spec> = overloads.iter().filter(|s| !s.arg_types.is_empty()).collect();
    let untyped: Vec<&Spec> = overloads.iter().filter(|s| s.arg_types.is_empty()).collect();
    let ordered: Vec<&Spec> = typed.into_iter().chain(untyped.into_iter()).collect();
    for spec in &ordered {
        if !spec.kwargs && !kwargs.is_empty() {
            continue;
        }
        let arg_count_ok = match spec.args {
            ArgSpec::Exact(n) => args.len() == n,
            ArgSpec::Min(n) => args.len() >= n,
            ArgSpec::Varargs => true,
        };
        if !arg_count_ok {
            continue;
        }
        let types_ok = spec.arg_types.iter().enumerate()
            .all(|(i, ty)| i >= args.len() || ty.matches(&args[i]));
        if !types_ok {
            continue;
        }
        return (spec.func)(args, kwargs);
    }
    if let Some(spec) = overloads.first() {
        if !spec.kwargs { assert_eq!(kwargs.len(), 0); }
        match spec.args {
            ArgSpec::Exact(n) => assert_eq!(args.len(), n),
            ArgSpec::Min(n) => assert!(args.len() >= n),
            ArgSpec::Varargs => {}
        }
        return (spec.func)(args, kwargs);
    }
    Primitive::Null
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