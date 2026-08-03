use crate::lang::primitive::{Primitive, truthy};
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;
use crate::lang::functions::Type;

pub fn switch(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    for (case, val) in kwargs {
        if truthy(&case) { return val; }
    }
    Primitive::Null
}
yaql_raw_function!("switch", switch, ArgSpec::Exact(0), [], true);

pub fn select_case(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut index = 0;
    for pred in args {
        if truthy(&pred) { return Primitive::Int(index); }
        index += 1;
    }
    Primitive::Int(index)
}
yaql_raw_function!("selectCase", select_case, ArgSpec::Varargs, [], false);

pub fn select_all_cases(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut cases = Vec::new();
    for (i, pred) in args.iter().enumerate() {
        if truthy(pred) { cases.push(Primitive::Int(i as i64)); }
    }
    Primitive::Array(cases)
}
yaql_raw_function!("selectAllCases", select_all_cases, ArgSpec::Varargs, [], false);

pub fn examine(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut cases = Vec::new();
    for pred in args {
        cases.push(Primitive::Boolean(truthy(&pred)));
    }
    Primitive::Array(cases)
}
yaql_raw_function!("examine", examine, ArgSpec::Varargs, [], false);

pub fn is_boolean(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    Primitive::Boolean(matches!(args[0], Primitive::Boolean(_)))
}
yaql_raw_function!("isBoolean", is_boolean, ArgSpec::Exact(1), [Type::Any], false);

pub fn coalesce(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    for arg in args {
        if !matches!(arg, Primitive::Null) { return arg; }
    }
    Primitive::Null
}
yaql_raw_function!("coalesce", coalesce, ArgSpec::Min(1), [], false);

pub fn concat(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut result = String::new();
    for arg in &args {
        if let Primitive::String(s) = arg { result.push_str(s); }
    }
    Primitive::String(result)
}
yaql_raw_function!("concat", concat, ArgSpec::Varargs, [], false);