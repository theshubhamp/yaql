use yaql_core::lang::{Primitive, truthy, primitive_eq};
use yaql_core::lang::primitive::LambdaBody;
use yaql_core::lang::functions::{EvalError, ArgSpec, Type};
use yaql_core::interpreter::eval_lambda;
use yaql_macros::yaql_function;

#[yaql_function("switch", ArgSpec::Exact(0), [], true)]
pub fn switch(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    for (case, val) in kwargs {
        if truthy(&case) { return Ok(val); }
    }
    Ok(Primitive::Null)
}
#[yaql_function("selectCase", ArgSpec::Varargs, [], false)]
pub fn select_case(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut index = 0;
    for pred in args {
        if truthy(&pred) { return Ok(Primitive::Int(index)); }
        index += 1;
    }
    Ok(Primitive::Int(index))
}
#[yaql_function("selectAllCases", ArgSpec::Varargs, [], false)]
pub fn select_all_cases(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut cases = Vec::new();
    for (i, pred) in args.iter().enumerate() {
        if truthy(pred) { cases.push(Primitive::Int(i as i64)); }
    }
    Ok(Primitive::Array(cases))
}
#[yaql_function("examine", ArgSpec::Varargs, [], false)]
pub fn examine(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut cases = Vec::new();
    for pred in args {
        cases.push(Primitive::Boolean(truthy(&pred)));
    }
    Ok(Primitive::Array(cases))
}
#[yaql_function("isBoolean")]
fn is_boolean(v: Any) -> bool {
    matches!(v.0, Primitive::Boolean(_))
}

#[yaql_function("coalesce", ArgSpec::Min(1), [], false)]
pub fn coalesce(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    for arg in args {
        if !matches!(arg, Primitive::Null) { return Ok(arg); }
    }
    Ok(Primitive::Null)
}
#[yaql_function("concat", ArgSpec::Varargs, [], false)]
pub fn concat(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut result = String::new();
    for arg in &args {
        if let Primitive::String(s) = arg { result.push_str(s); }
    }
    Ok(Primitive::String(result))
}
#[yaql_function("concat", ArgSpec::Varargs, [Type::Array], false)]
pub fn concat_arrays(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut result = Vec::new();
    for arg in &args {
        if let Primitive::Array(arr) = arg { result.extend(arr.iter().cloned()); }
    }
    Ok(Primitive::Array(result))
}
#[yaql_function("switchCase", ArgSpec::Min(1), [Type::Int], false)]
pub fn switch_case_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let Primitive::Int(index) = &args[0] else { return Ok(Primitive::Null) };
    let cases = &args[1..];
    if cases.is_empty() { return Ok(Primitive::Null); }
    let idx = (*index as usize).min(cases.len() - 1);
    Ok(cases[idx].clone())
}
// let(name => value, ...) -> Map of bindings
#[yaql_function("let", ArgSpec::Exact(0), [], true)]
pub fn let_fn(_args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut map = std::collections::HashMap::new();
    for (k, v) in kwargs {
        if let Primitive::String(key) = k {
            map.insert(key, v);
        }
    }
    Ok(Primitive::Map(map))
}
// let(value) -> value (for let($.memorize()) -> ...)
#[yaql_function("let", ArgSpec::Min(1), [Type::Any], false)]
pub fn let_value_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    if args.is_empty() { Ok(Primitive::Null) } else { Ok(args[0].clone()) }
}
// with(value) -> value
#[yaql_function("with", ArgSpec::Min(1), [Type::Any], false)]
pub fn with_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    if args.is_empty() { Ok(Primitive::Null) } else { Ok(args[0].clone()) }
}
// memorize: just returns the value (no lazy eval in our impl)
#[yaql_function("memorize")]
fn memorize(v: Any) -> Primitive { v.0 }

// generateMany: graph traversal
#[yaql_function("generateMany", ArgSpec::Min(2), [Type::Any, Type::Any], true)]
pub fn generate_many_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let start = &args[0];
    let Some(neighbors_lambda) = get_lambda(&args, 1) else { return Ok(Primitive::Null) };
    let decycle = kwargs.iter().any(|(k, _)| {
        if let Primitive::String(key) = k { key == "decycle" } else { false }
    });
    let depth_first = kwargs.iter().any(|(k, _)| {
        if let Primitive::String(key) = k { key == "depthFirst" } else { false }
    });
    let mut visited = Vec::new();
    let mut result = Vec::new();
    if depth_first {
        let mut stack = vec![start.clone()];
        while let Some(node) = stack.pop() {
            if visited.iter().any(|v| primitive_eq(v, &node)) { continue; }
            if decycle { visited.push(node.clone()); }
            result.push(node.clone());
            let neighbors = eval_lambda(neighbors_lambda, node)?;
            if let Primitive::Array(nbrs) = neighbors {
                for n in nbrs.iter().rev() {
                    if !decycle || !visited.iter().any(|v| primitive_eq(v, n)) {
                        stack.push(n.clone());
                    }
                }
            }
        }
    } else {
        let mut queue = vec![start.clone()];
        let mut head = 0;
        while head < queue.len() {
            let node = queue[head].clone();
            head += 1;
            if visited.iter().any(|v| primitive_eq(v, &node)) { continue; }
            if decycle { visited.push(node.clone()); }
            result.push(node.clone());
            let neighbors = eval_lambda(neighbors_lambda, node)?;
            if let Primitive::Array(nbrs) = neighbors {
                for n in nbrs {
                    if !decycle || !visited.iter().any(|v| primitive_eq(v, &n)) {
                        queue.push(n);
                    }
                }
            }
        }
    }
    Ok(Primitive::Array(result))
}
fn get_lambda(args: &[Primitive], idx: usize) -> Option<&LambdaBody> {
    if let Some(Primitive::Lambda(l)) = args.get(idx) { Some(l) } else { None }
}
