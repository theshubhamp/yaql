use crate::lang::primitive::{Primitive, compare, primitive_eq};
use crate::lang::operators::eq;
use crate::lang::functions::{FromPrimitive, IntoPrimitive, Spec};
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;
use crate::lang::functions::Type;
use std::collections::HashMap;

yaql_function!("get", get_fn(m: HashMap<String, Primitive>, key: String) -> Primitive {
    m.get(&key).cloned().unwrap_or(Primitive::Null)
});
yaql_function!("keys", keys_fn(m: HashMap<String, Primitive>) -> Vec<Primitive> {
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    keys.into_iter().map(Primitive::String).collect()
});
yaql_function!("values", values_fn(m: HashMap<String, Primitive>) -> Vec<Primitive> {
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    keys.into_iter().filter_map(|k| m.get(&k).cloned()).collect()
});

pub fn list_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    Primitive::Array(args)
}
yaql_raw_function!("list", list_fn, ArgSpec::Varargs, [], false);

pub fn dict_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut map = HashMap::new();
    for (k, v) in kwargs {
        let key = match k {
            Primitive::String(s) => s,
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => continue,
        };
        map.insert(key, v);
    }
    Primitive::Map(map)
}
yaql_raw_function!("dict", dict_fn, ArgSpec::Varargs, [], true);

pub fn contains_array(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Boolean(false) };
    Primitive::Boolean(arr.iter().any(|e| primitive_eq(e, &args[1])))
}
yaql_raw_function!("contains", contains_array, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

pub fn contains_string(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Boolean(false) };
    let Primitive::String(sub) = &args[1] else { return Primitive::Boolean(false) };
    Primitive::Boolean(s.contains(sub.as_str()))
}
yaql_raw_function!("contains", contains_string, ArgSpec::Exact(2), [Type::String, Type::String], false);

pub fn set_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut seen = Vec::new();
    for arg in args {
        if !seen.iter().any(|e: &Primitive| matches!(eq(e.clone(), arg.clone()), Primitive::Boolean(true))) {
            seen.push(arg);
        }
    }
    Primitive::Array(seen)
}
yaql_raw_function!("set", set_fn, ArgSpec::Varargs, [], false);

pub fn max(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let iter: Vec<Primitive> = if args.len() == 1 {
        match &args[0] { Primitive::Array(a) => a.clone(), other => vec![other.clone()] }
    } else { args };
    let mut result: Option<Primitive> = None;
    for arg in iter {
        result = Some(match (result, arg) {
            (None, a) | (Some(Primitive::Null), a) => a,
            (r, Primitive::Null) => r.unwrap(),
            (Some(r), a) => if compare(&r, &a) >= std::cmp::Ordering::Equal { r } else { a },
        });
    }
    result.unwrap_or(Primitive::Null)
}
yaql_raw_function!("max", max, ArgSpec::Min(1), [], false);

pub fn min(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let iter: Vec<Primitive> = if args.len() == 1 {
        match &args[0] { Primitive::Array(a) => a.clone(), other => vec![other.clone()] }
    } else { args };
    let mut result: Option<Primitive> = None;
    for arg in iter {
        result = Some(match (result, arg) {
            (None, a) => a,
            (Some(Primitive::Null), _) => Primitive::Null,
            (r, Primitive::Null) => r.unwrap(),
            (Some(r), a) => if compare(&r, &a) <= std::cmp::Ordering::Equal { r } else { a },
        });
    }
    result.unwrap_or(Primitive::Null)
}
yaql_raw_function!("min", min, ArgSpec::Min(1), [], false);