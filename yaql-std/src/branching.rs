use yaql_core::lang::{Primitive, truthy, primitive_eq};
use yaql_core::lang::primitive::LambdaBody;
use yaql_core::lang::functions::{EvalError, ArgSpec, Type};
use yaql_core::interpreter::eval_lambda;
use yaql_macros::yaql_function;
use crate::yaql_raw_function;

pub fn switch(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    for (case, val) in kwargs {
        if truthy(&case) { return Ok(val); }
    }
    Ok(Primitive::Null)
}
yaql_raw_function!("switch", switch, ArgSpec::Exact(0), [], true);

pub fn select_case(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut index = 0;
    for pred in args {
        if truthy(&pred) { return Ok(Primitive::Int(index)); }
        index += 1;
    }
    Ok(Primitive::Int(index))
}
yaql_raw_function!("selectCase", select_case, ArgSpec::Varargs, [], false);

pub fn select_all_cases(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut cases = Vec::new();
    for (i, pred) in args.iter().enumerate() {
        if truthy(pred) { cases.push(Primitive::Int(i as i64)); }
    }
    Ok(Primitive::Array(cases))
}
yaql_raw_function!("selectAllCases", select_all_cases, ArgSpec::Varargs, [], false);

pub fn examine(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut cases = Vec::new();
    for pred in args {
        cases.push(Primitive::Boolean(truthy(&pred)));
    }
    Ok(Primitive::Array(cases))
}
yaql_raw_function!("examine", examine, ArgSpec::Varargs, [], false);

#[yaql_function("isBoolean")]
fn is_boolean(v: Any) -> bool {
    matches!(v.0, Primitive::Boolean(_))
}

pub fn coalesce(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    for arg in args {
        if !matches!(arg, Primitive::Null) { return Ok(arg); }
    }
    Ok(Primitive::Null)
}
yaql_raw_function!("coalesce", coalesce, ArgSpec::Min(1), [], false);

pub fn concat(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut result = String::new();
    for arg in &args {
        if let Primitive::String(s) = arg { result.push_str(s); }
    }
    Ok(Primitive::String(result))
}
yaql_raw_function!("concat", concat, ArgSpec::Varargs, [], false);

pub fn concat_arrays(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut result = Vec::new();
    for arg in &args {
        if let Primitive::Array(arr) = arg { result.extend(arr.iter().cloned()); }
    }
    Ok(Primitive::Array(result))
}
yaql_raw_function!("concat", concat_arrays, ArgSpec::Varargs, [Type::Array], false);

pub fn switch_case_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let Primitive::Int(index) = &args[0] else { return Ok(Primitive::Null) };
    let cases = &args[1..];
    if cases.is_empty() { return Ok(Primitive::Null); }
    let idx = (*index as usize).min(cases.len() - 1);
    Ok(cases[idx].clone())
}
yaql_raw_function!("switchCase", switch_case_fn, ArgSpec::Min(1), [Type::Int], false);

// let(name => value, ...) -> Map of bindings
pub fn let_fn(_args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut map = std::collections::HashMap::new();
    for (k, v) in kwargs {
        if let Primitive::String(key) = k {
            map.insert(key, v);
        }
    }
    Ok(Primitive::Map(map))
}
yaql_raw_function!("let", let_fn, ArgSpec::Exact(0), [], true);

// let(value) -> value (for let($.memorize()) -> ...)
pub fn let_value_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    if args.is_empty() { Ok(Primitive::Null) } else { Ok(args[0].clone()) }
}
yaql_raw_function!("let", let_value_fn, ArgSpec::Min(1), [Type::Any], false);

// with(value) -> value
pub fn with_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    if args.is_empty() { Ok(Primitive::Null) } else { Ok(args[0].clone()) }
}
yaql_raw_function!("with", with_fn, ArgSpec::Min(1), [Type::Any], false);

// memorize: just returns the value (no lazy eval in our impl)
#[yaql_function("memorize")]
fn memorize(v: Any) -> Primitive { v.0 }

// generateMany: graph traversal
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
yaql_raw_function!("generateMany", generate_many_fn, ArgSpec::Min(2), [Type::Any, Type::Any], true);

fn get_lambda(args: &[Primitive], idx: usize) -> Option<&LambdaBody> {
    if let Some(Primitive::Lambda(l)) = args.get(idx) { Some(l) } else { None }
}
