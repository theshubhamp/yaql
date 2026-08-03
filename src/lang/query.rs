use crate::lang::primitive::Primitive;
use crate::lang::functions::{FromPrimitive, IntoPrimitive, Any, Spec};
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;
use crate::lang::functions::Type;

yaql_function!("skip", skip(arr: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    let n = (n as usize).min(arr.len());
    arr[n..].to_vec()
});
yaql_function!("take", take(arr: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    let n = (n as usize).min(arr.len());
    arr[..n].to_vec()
});
yaql_raw_function!("limit", take::func, ArgSpec::Exact(2), [Type::Array, Type::Int], false);

yaql_function!("count", count_array(arr: Vec<Primitive>) -> i64 { arr.len() as i64 });
yaql_function!("count", count_map(m: std::collections::HashMap<String, Primitive>) -> i64 { m.len() as i64 });
yaql_function!("count", count_string(s: String) -> i64 { s.chars().count() as i64 });

yaql_function!("first", first(arr: Vec<Primitive>) -> Option<Primitive> { arr.first().cloned() });
yaql_function!("last", last(arr: Vec<Primitive>) -> Option<Primitive> { arr.last().cloned() });

yaql_function!("range", range_one(n: i64) -> Vec<Primitive> { (0..n).map(Primitive::Int).collect() });
yaql_function!("range", range_two(start: i64, end: i64) -> Vec<Primitive> { (start..end).map(Primitive::Int).collect() });

pub fn append(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Some(Primitive::Array(arr)) = args.first() else { return Primitive::Null };
    let mut result = arr.clone();
    result.extend(args[1..].iter().cloned());
    Primitive::Array(result)
}
yaql_raw_function!("append", append, ArgSpec::Min(1), [Type::Array], false);

yaql_function!("reverse", reverse_array(arr: Vec<Primitive>) -> Vec<Primitive> {
    let mut r = arr; r.reverse(); r
});
yaql_function!("reverse", reverse_string(s: String) -> String { s.chars().rev().collect() });

yaql_function!("isIterable", is_iterable(v: Any) -> bool {
    matches!(v.0, Primitive::Array(_) | Primitive::Map(_) | Primitive::String(_))
});

yaql_function!("distinct", distinct(arr: Vec<Primitive>) -> Vec<Primitive> {
    let mut seen = Vec::new();
    for e in &arr {
        if !seen.iter().any(|s: &Primitive| matches!(crate::lang::operators::eq(s.clone(), e.clone()), Primitive::Boolean(true))) {
            seen.push(e.clone());
        }
    }
    seen
});

pub fn sum(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Some(Primitive::Array(arr)) = args.first() else { return Primitive::Null };
    let init = if args.len() > 1 { args[1].clone() } else { Primitive::Int(0) };
    arr.iter().fold(init, |acc, e| crate::lang::operators::add(acc, e.clone()))
}
yaql_raw_function!("sum", sum, ArgSpec::Min(1), [Type::Array], false);