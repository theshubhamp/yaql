use yaql_core::lang::{Primitive, truthy, primitive_eq};
use yaql_core::lang::primitive::LambdaBody;
use yaql_core::lang::functions::{EvalError, Varargs, Kwargs, Any};
use yaql_core::interpreter::eval_lambda;
use yaql_macros::yaql_function;

#[yaql_function("switch")]
pub fn switch(kwargs: Kwargs) -> Primitive {
    for (case, val) in kwargs.0 {
        if truthy(&case) { return val; }
    }
    Primitive::Null
}
#[yaql_function("selectCase")]
pub fn select_case(args: Varargs<0>) -> Primitive {
    let mut index = 0;
    for pred in args.0 {
        if truthy(&pred) { return Primitive::Int(index); }
        index += 1;
    }
    Primitive::Int(index)
}
#[yaql_function("selectAllCases")]
pub fn select_all_cases(args: Varargs<0>) -> Primitive {
    let mut cases = Vec::new();
    for (i, pred) in args.0.iter().enumerate() {
        if truthy(pred) { cases.push(Primitive::Int(i as i64)); }
    }
    Primitive::Array(cases)
}
#[yaql_function("examine")]
pub fn examine(args: Varargs<0>) -> Primitive {
    let mut cases = Vec::new();
    for pred in args.0 {
        cases.push(Primitive::Boolean(truthy(&pred)));
    }
    Primitive::Array(cases)
}
#[yaql_function("isBoolean")]
fn is_boolean(v: Any) -> bool {
    matches!(v.0, Primitive::Boolean(_))
}

#[yaql_function("coalesce")]
pub fn coalesce(args: Varargs<1>) -> Primitive {
    for arg in args.0 {
        if !matches!(arg, Primitive::Null) { return arg; }
    }
    Primitive::Null
}
#[yaql_function("concat")]
pub fn concat(args: Varargs<0>) -> Primitive {
    let mut result = String::new();
    for arg in &args.0 {
        if let Primitive::String(s) = arg { result.push_str(s); }
    }
    Primitive::String(result)
}
#[yaql_function("concat")]
pub fn concat_arrays(first: Vec<Primitive>, rest: Varargs<0>) -> Primitive {
    let mut result = first;
    for arg in &rest.0 {
        if let Primitive::Array(arr) = arg { result.extend(arr.iter().cloned()); }
    }
    Primitive::Array(result)
}
#[yaql_function("switchCase")]
pub fn switch_case_fn(index: i64, cases: Varargs<0>) -> Primitive {
    if cases.0.is_empty() { return Primitive::Null; }
    let idx = (index as usize).min(cases.0.len() - 1);
    cases.0[idx].clone()
}
// let(name => value, ...) -> Map of bindings
#[yaql_function("let")]
pub fn let_fn(kwargs: Kwargs) -> Primitive {
    let mut map = std::collections::HashMap::new();
    for (k, v) in kwargs.0 {
        if let Primitive::String(key) = k {
            map.insert(key, v);
        }
    }
    Primitive::Map(map)
}
// let(value) -> value (for let($.memorize()) -> ...)
#[yaql_function("let")]
pub fn let_value_fn(args: Varargs<1>) -> Primitive {
    if args.0.is_empty() { Primitive::Null } else { args.0[0].clone() }
}
// with(value) -> value
#[yaql_function("with")]
pub fn with_fn(args: Varargs<1>) -> Primitive {
    if args.0.is_empty() { Primitive::Null } else { args.0[0].clone() }
}
// memorize: just returns the value (no lazy eval in our impl)
#[yaql_function("memorize")]
fn memorize(v: Any) -> Primitive { v.0 }

// generateMany: graph traversal
#[yaql_function("generateMany")]
pub fn generate_many_fn(start: Any, neighbors: LambdaBody, _rest: Varargs<0>, kwargs: Kwargs) -> Result<Primitive, EvalError> {
    let decycle = kwargs.0.iter().any(|(k, _)| {
        if let Primitive::String(key) = k { key == "decycle" } else { false }
    });
    let depth_first = kwargs.0.iter().any(|(k, _)| {
        if let Primitive::String(key) = k { key == "depthFirst" } else { false }
    });
    let mut visited = Vec::new();
    let mut result = Vec::new();
    if depth_first {
        let mut stack = vec![start.0.clone()];
        while let Some(node) = stack.pop() {
            if visited.iter().any(|v| primitive_eq(v, &node)) { continue; }
            if decycle { visited.push(node.clone()); }
            result.push(node.clone());
            let neighbors = eval_lambda(&neighbors, node)?;
            if let Primitive::Array(nbrs) = neighbors {
                for n in nbrs.iter().rev() {
                    if !decycle || !visited.iter().any(|v| primitive_eq(v, n)) {
                        stack.push(n.clone());
                    }
                }
            }
        }
    } else {
        let mut queue = vec![start.0.clone()];
        let mut head = 0;
        while head < queue.len() {
            let node = queue[head].clone();
            head += 1;
            if visited.iter().any(|v| primitive_eq(v, &node)) { continue; }
            if decycle { visited.push(node.clone()); }
            result.push(node.clone());
            let neighbors = eval_lambda(&neighbors, node)?;
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
