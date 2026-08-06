use crate::lang::primitive::Primitive;
use crate::lang::functions::{FromPrimitive, IntoPrimitive, Any, Spec, SetVec};
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;
use crate::lang::functions::Type;

yaql_function!("skip", skip(arr: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    let n = (n as usize).min(arr.len());
    arr[n..].to_vec()
});
yaql_function!("skip", skip_set(arr: SetVec, n: i64) -> SetVec {
    let n = (n as usize).min(arr.0.len());
    SetVec(arr.0[n..].to_vec())
});
yaql_function!("take", take(arr: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    let n = (n as usize).min(arr.len());
    arr[..n].to_vec()
});
yaql_function!("take", take_set(arr: SetVec, n: i64) -> SetVec {
    let n = (n as usize).min(arr.0.len());
    SetVec(arr.0[..n].to_vec())
});
yaql_raw_function!("limit", take::func, ArgSpec::Exact(2), [Type::Array, Type::Int], false);

yaql_function!("count", count_array(arr: Vec<Primitive>) -> i64 { arr.len() as i64 });
yaql_function!("count", count_set(arr: SetVec) -> i64 { arr.0.len() as i64 });
yaql_function!("count", count_map(m: std::collections::HashMap<String, Primitive>) -> i64 { m.len() as i64 });
yaql_function!("count", count_string(s: String) -> i64 { s.chars().count() as i64 });

yaql_function!("first", first(arr: Vec<Primitive>) -> Option<Primitive> { arr.first().cloned() });
yaql_function!("first", first_set(arr: SetVec) -> Option<Primitive> { arr.0.first().cloned() });
yaql_function!("last", last(arr: Vec<Primitive>) -> Option<Primitive> { arr.last().cloned() });
yaql_function!("last", last_set(arr: SetVec) -> Option<Primitive> { arr.0.last().cloned() });

pub fn first_default_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    arr.first().cloned().unwrap_or_else(|| args[1].clone())
}
yaql_raw_function!("first", first_default_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);
yaql_raw_function!("first", first_default_fn, ArgSpec::Exact(2), [Type::Set, Type::Any], false);

pub fn last_default_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    arr.last().cloned().unwrap_or_else(|| args[1].clone())
}
yaql_raw_function!("last", last_default_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);
yaql_raw_function!("last", last_default_fn, ArgSpec::Exact(2), [Type::Set, Type::Any], false);

yaql_function!("range", range_one(n: i64) -> Vec<Primitive> { (0..n).map(Primitive::Int).collect() });
yaql_function!("range", range_two(start: i64, end: i64) -> Vec<Primitive> { (start..end).map(Primitive::Int).collect() });

pub fn range_three(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Int(start) = args[0] else { return Primitive::Null };
    let Primitive::Int(end) = args[1] else { return Primitive::Null };
    let Primitive::Int(step) = args[2] else { return Primitive::Null };
    let result: Vec<Primitive> = if step > 0 {
        (0..).map(|i| start + i * step).take_while(|&x| x < end).map(Primitive::Int).collect()
    } else if step < 0 {
        (0..).map(|i| start + i * step).take_while(|&x| x > end).map(Primitive::Int).collect()
    } else {
        return Primitive::Null;
    };
    Primitive::Array(result)
}
yaql_raw_function!("range", range_three, ArgSpec::Exact(3), [Type::Int, Type::Int, Type::Int], false);

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
yaql_function!("reverse", reverse_set(arr: SetVec) -> SetVec {
    let mut r = arr.0; r.reverse(); SetVec(r)
});
yaql_function!("reverse", reverse_string(s: String) -> String { s.chars().rev().collect() });

yaql_function!("isIterable", is_iterable(v: Any) -> bool {
    matches!(v.0, Primitive::Array(_) | Primitive::Set(_))
});

yaql_function!("distinct", distinct(arr: Vec<Primitive>) -> Vec<Primitive> {
    let mut seen = Vec::new();
    for e in &arr {
        crate::lang::sets::set_push_unique(&mut seen, e);
    }
    seen
});

pub fn sum(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let arr: Vec<Primitive> = match args.first() {
        Some(Primitive::Array(arr)) => arr.clone(),
        Some(Primitive::Set(arr)) => arr.clone(),
        _ => return Primitive::Null,
    };
    let init = if args.len() > 1 { args[1].clone() } else { Primitive::Int(0) };
    arr.iter().fold(init, |acc, e| crate::lang::operators::add(acc, e.clone()))
}
yaql_raw_function!("sum", sum, ArgSpec::Min(1), [Type::Array], false);
yaql_raw_function!("sum", sum, ArgSpec::Min(1), [Type::Set], false);

pub fn split_at_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let pos = match &args[1] {
        Primitive::Int(n) => {
            let len = arr.len() as i64;
            if *n < 0 { ((len + n).max(0)) as usize } else { (*n as usize).min(arr.len()) }
        }
        _ => return Primitive::Null,
    };
    Primitive::Array(vec![
        Primitive::Array(arr[..pos].to_vec()),
        Primitive::Array(arr[pos..].to_vec()),
    ])
}
yaql_raw_function!("splitAt", split_at_fn, ArgSpec::Exact(2), [Type::Array, Type::Int], false);

// --- enumerate ---
pub fn enumerate_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let start = if args.len() > 1 {
        if let Primitive::Int(n) = &args[1] { *n } else { 0 }
    } else { 0 };
    let result: Vec<Primitive> = arr.iter().enumerate().map(|(i, v)| {
        Primitive::Array(vec![Primitive::Int(start + i as i64), v.clone()])
    }).collect();
    Primitive::Array(result)
}
yaql_raw_function!("enumerate", enumerate_fn, ArgSpec::Min(1), [Type::Array], false);

// --- single ---
pub fn single_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    if arr.len() == 1 { arr[0].clone() } else { Primitive::Null }
}
yaql_raw_function!("single", single_fn, ArgSpec::Exact(1), [Type::Array], false);

// --- slice (chunk into sublists of given size) ---
pub fn slice_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Primitive::Int(size) = &args[1] else { return Primitive::Null };
    let size = (*size as usize).max(1);
    let result: Vec<Primitive> = arr.chunks(size).map(|chunk| Primitive::Array(chunk.to_vec())).collect();
    Primitive::Array(result)
}
yaql_raw_function!("slice", slice_fn, ArgSpec::Exact(2), [Type::Array, Type::Int], false);

// --- any (no predicate: non-empty) ---
pub fn any_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    Primitive::Boolean(!arr.is_empty())
}
yaql_raw_function!("any", any_fn, ArgSpec::Exact(1), [Type::Array], false);

// --- all (no predicate: all truthy) ---
pub fn all_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    Primitive::Boolean(arr.iter().all(crate::lang::truthy))
}
yaql_raw_function!("all", all_fn, ArgSpec::Exact(1), [Type::Array], false);

// --- defaultIfEmpty ---
pub fn default_if_empty_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    match &args[0] {
        Primitive::Array(arr) if arr.is_empty() => args[1].clone(),
        other => other.clone(),
    }
}
yaql_raw_function!("defaultIfEmpty", default_if_empty_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);