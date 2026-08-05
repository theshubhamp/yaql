use std::collections::HashMap;
use crate::ast::{Value, Visitor};
use crate::lang::{truthy, ArgSpec, BINARY_OPERATORS, FUNCTIONS, Primitive, Spec};

pub struct Interpreter {
    pub context: Primitive,
}

impl Visitor<Option<Primitive>> for Interpreter {
    fn visit(&self, value: Value) -> Option<Primitive> {
        return match value {
            Value::StringLiteral(string) => self.visit_string_literal(string),
            Value::IntLiteral(num) => self.visit_int_literal(num),
            Value::FloatLiteral(num) => self.visit_float_literal(num),
            Value::BooleanLiteral(bool) => self.visit_boolean_literal(bool),
            Value::NullLiteral => self.visit_null_literal(),
            Value::Dollar(path) => self.visit_dollar(path),
            Value::FunctionCall(identifier, args, kwargs) => self.visit_function_call(identifier, args, kwargs),
            Value::BinaryOperator(left, op, right) => self.visit_binary_operator(*left, op, *right),
            Value::UnaryOperator(op, operand) => self.visit_unary_operator(op, *operand),
            Value::MethodCall(receiver, optional, method, args, kwargs) => self.visit_method_call(*receiver, optional, method, args, kwargs),
            Value::List(elements) => self.visit_list(elements),
            Value::Dict(entries) => self.visit_dict(entries),
            Value::Index(collection, indices) => self.visit_index(*collection, indices),
        }
    }

    fn visit_string_literal(&self, string: String) -> Option<Primitive> {
        return Some(Primitive::String(unquote(&string)));
    }

    fn visit_int_literal(&self, num: i64) -> Option<Primitive> {
        return Some(Primitive::Int(num));
    }

    fn visit_float_literal(&self, num: f64) -> Option<Primitive> {
        return Some(Primitive::Float(num));
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
        let overloads = FUNCTIONS.lookup(identifier);
        let arg_values: Vec<Primitive> = args.into_iter().filter_map(|a| self.visit(a)).collect();
        let kwarg_values: Vec<(Primitive, Primitive)> = kwargs.into_iter()
            .filter_map(|(k, v)| {
                let k = self.visit(k)?;
                let v = self.visit(v)?;
                Some((k, v))
            })
            .collect();
        Some(dispatch(overloads, arg_values, kwarg_values))
    }

    fn visit_binary_operator(&self, left: Value, op: String, right: Value) -> Option<Primitive> {
        let left_value = self.visit(left)?;
        let right_value = self.visit(right)?;
        Some(BINARY_OPERATORS.lookup(op)(left_value, right_value))
    }

    fn visit_method_call(&self, receiver: Value, optional: bool, method: String, args: Vec<Value>, kwargs: Vec<(Value, Value)>) -> Option<Primitive> {
        let receiver_value = self.visit(receiver)?;

        if optional && matches!(receiver_value, Primitive::Null) {
            return Some(Primitive::Null);
        }

        let overloads = FUNCTIONS.lookup(method);
        let mut arg_values = vec![receiver_value];
        for arg in args {
            arg_values.push(self.visit(arg)?);
        }
        let kwarg_values: Vec<(Primitive, Primitive)> = kwargs.into_iter()
            .filter_map(|(k, v)| {
                let k = self.visit(k)?;
                let v = self.visit(v)?;
                Some((k, v))
            })
            .collect();
        Some(dispatch(overloads, arg_values, kwarg_values))
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
}

/// Find the best matching overload and call it.
/// Overloads are sorted by specificity (exact type matches first, Any/Number last).
fn dispatch(overloads: Vec<Spec>, args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut overloads = overloads;
    // Sort by specificity score (lower = more specific = tried first)
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
    // No overload matched — try to find one for error reporting
    if let Some(spec) = overloads.first() {
        // Validate against first spec for a useful panic message
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
    // Only process if surrounded by matching quote characters.
    if !((first == '\'' || first == '"' || first == '`') && first == last) {
        return raw.to_string();
    }
    let inner = &raw[1..raw.len() - 1];
    if first == '`' {
        // Verbatim: only \` is escaped, everything else is literal.
        inner.replace("\\`", "`")
    } else if first == '\'' {
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
