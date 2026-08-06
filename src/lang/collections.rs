use crate::lang::primitive::{Primitive, compare, primitive_eq};
use crate::lang::functions::{FromPrimitive, IntoPrimitive, Any, Spec};
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;
use crate::lang::functions::Type;
use std::collections::HashMap;

yaql_function!("get", get_fn(m: HashMap<String, Primitive>, key: String) -> Primitive {
    m.get(&key).cloned().unwrap_or(Primitive::Null)
});
yaql_function!("get", get_default(m: HashMap<String, Primitive>, key: String, default: Any) -> Primitive {
    m.get(&key).cloned().unwrap_or(default.0)
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

yaql_function!("items", items_fn(m: HashMap<String, Primitive>) -> Vec<Primitive> {
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    keys.into_iter().map(|k| {
        let v = m.get(&k).cloned().unwrap_or(Primitive::Null);
        Primitive::Array(vec![Primitive::String(k), v])
    }).collect()
});

yaql_function!("containsKey", contains_key_str(m: HashMap<String, Primitive>, key: String) -> bool {
    m.contains_key(&key)
});
yaql_function!("containsKey", contains_key_any(m: HashMap<String, Primitive>, _key: Any) -> bool {
    false
});

pub fn list_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    Primitive::Array(args)
}
yaql_raw_function!("list", list_fn, ArgSpec::Varargs, [], false);

yaql_function!("toList", to_list_fn(a: Vec<Primitive>) -> Vec<Primitive> { a });

pub fn dict_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut map = HashMap::new();
    let pairs: Vec<Primitive> = if args.len() == 1 {
        if let Primitive::Array(a) = &args[0] { a.clone() } else { args }
    } else { args };
    for arg in &pairs {
        if let Primitive::Array(pair) = arg {
            if pair.len() == 2 {
                let key = match &pair[0] {
                    Primitive::String(s) => s.clone(),
                    Primitive::Int(n) => n.to_string(),
                    Primitive::Boolean(b) => b.to_string(),
                    Primitive::Null => "null".to_string(),
                    _ => continue,
                };
                map.insert(key, pair[1].clone());
            }
        }
    }
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

pub fn dict_set_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    if args.len() <= 1 && kwargs.is_empty() {
        return crate::lang::sets::set_fn(args, kwargs);
    }
    let Primitive::Map(mut m) = args[0].clone() else { return Primitive::Null };
    if args.len() == 3 {
        let key = match &args[1] {
            Primitive::String(s) => s.clone(),
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => return Primitive::Null,
        };
        m.insert(key, args[2].clone());
    } else if args.len() == 2 {
        if let Primitive::Map(other) = &args[1] {
            for (k, v) in other {
                m.insert(k.clone(), v.clone());
            }
        }
    }
    for (k, v) in kwargs {
        let key = match k {
            Primitive::String(s) => s,
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => continue,
        };
        m.insert(key, v);
    }
    Primitive::Map(m)
}
yaql_raw_function!("set", dict_set_fn, ArgSpec::Min(1), [Type::Map], true);

pub fn dict_delete_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Map(mut m) = args[0].clone() else { return Primitive::Null };
    for arg in &args[1..] {
        let key = match arg {
            Primitive::String(s) => s.clone(),
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => continue,
        };
        m.remove(&key);
    }
    Primitive::Map(m)
}
yaql_raw_function!("delete", dict_delete_fn, ArgSpec::Min(2), [Type::Map], false);

pub fn dict_delete_all_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Map(mut m) = args[0].clone() else { return Primitive::Null };
    if let Primitive::Array(keys) = &args[1] {
        for k in keys {
            let key = match k {
                Primitive::String(s) => s.clone(),
                Primitive::Int(n) => n.to_string(),
                Primitive::Boolean(b) => b.to_string(),
                Primitive::Null => "null".to_string(),
                _ => continue,
            };
            m.remove(&key);
        }
    }
    Primitive::Map(m)
}
yaql_raw_function!("deleteAll", dict_delete_all_fn, ArgSpec::Exact(2), [Type::Map, Type::Array], false);

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

pub fn max(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let iter: Vec<Primitive> = if args.len() == 1 {
        match &args[0] { Primitive::Array(a) => a.clone(), Primitive::Set(a) => a.clone(), other => vec![other.clone()] }
    } else if args.len() == 2 && matches!(&args[0], Primitive::Array(_) | Primitive::Set(_)) {
        // Method form: collection.max(default) — return default if empty
        let coll = match &args[0] { Primitive::Array(a) => a.clone(), Primitive::Set(a) => a.clone(), _ => vec![] };
        if coll.is_empty() { return args[1].clone(); }
        coll
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
        match &args[0] { Primitive::Array(a) => a.clone(), Primitive::Set(a) => a.clone(), other => vec![other.clone()] }
    } else if args.len() == 2 && matches!(&args[0], Primitive::Array(_) | Primitive::Set(_)) {
        let coll = match &args[0] { Primitive::Array(a) => a.clone(), Primitive::Set(a) => a.clone(), _ => vec![] };
        if coll.is_empty() { return args[1].clone(); }
        coll
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

// --- List mutation ---

fn norm_idx(i: i64, len: usize) -> usize {
    if i < 0 { ((len as i64) + i).max(0) as usize } else { (i as usize).min(len) }
}

pub fn list_delete(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let start = match &args[1] { Primitive::Int(n) => norm_idx(*n, arr.len()), _ => return Primitive::Null };
    let count = if args.len() > 2 {
        match &args[2] {
            Primitive::Int(n) if *n < 0 => arr.len() - start,
            Primitive::Int(n) => *n as usize,
            _ => 1,
        }
    } else { 1 };
    let mut result = arr.clone();
    let end = (start + count).min(result.len());
    result.drain(start..end);
    Primitive::Array(result)
}
yaql_raw_function!("delete", list_delete, ArgSpec::Min(2), [Type::Array, Type::Int], false);

pub fn list_insert(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let mut pos = match &args[1] { Primitive::Int(n) => norm_idx(*n, arr.len()), _ => return Primitive::Null };
    pos = pos.min(arr.len());
    let mut result = arr.clone();
    result.insert(pos, args[2].clone());
    Primitive::Array(result)
}
yaql_raw_function!("insert", list_insert, ArgSpec::Exact(3), [Type::Array, Type::Int, Type::Any], false);

pub fn list_insert_many(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let pos = match &args[1] {
        Primitive::Int(n) if *n < 0 => 0,
        Primitive::Int(n) => (*n as usize).min(arr.len()),
        _ => return Primitive::Null,
    };
    let Primitive::Array(items) = &args[2] else { return Primitive::Null };
    let mut result = arr.clone();
    for (i, item) in items.iter().enumerate() {
        result.insert(pos + i, item.clone());
    }
    Primitive::Array(result)
}
yaql_raw_function!("insertMany", list_insert_many, ArgSpec::Exact(3), [Type::Array, Type::Int, Type::Array], false);

pub fn list_replace(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let start = match &args[1] { Primitive::Int(n) => norm_idx(*n, arr.len()), _ => return Primitive::Null };
    let value = args[2].clone();
    let count = if args.len() > 3 {
        match &args[3] {
            Primitive::Int(n) if *n < 0 => arr.len() - start,
            Primitive::Int(n) => *n as usize,
            _ => 1,
        }
    } else { 1 };
    let mut result = arr.clone();
    let end = (start + count).min(result.len());
    if count == 1 {
        if start < result.len() { result[start] = value; }
    } else {
        result.drain(start..end);
        result.insert(start, value);
    }
    Primitive::Array(result)
}
yaql_raw_function!("replace", list_replace, ArgSpec::Min(3), [Type::Array, Type::Int, Type::Any], false);

pub fn list_replace_many(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let start = match &args[1] { Primitive::Int(n) => norm_idx(*n, arr.len()), _ => return Primitive::Null };
    let Primitive::Array(items) = &args[2] else { return Primitive::Null };
    let count = if args.len() > 3 {
        match &args[3] {
            Primitive::Int(n) if *n < 0 => arr.len() - start,
            Primitive::Int(n) => *n as usize,
            _ => 1,
        }
    } else { 1 };
    let mut result = arr.clone();
    let end = (start + count).min(result.len());
    result.drain(start..end);
    for (i, item) in items.iter().enumerate() {
        result.insert(start + i, item.clone());
    }
    Primitive::Array(result)
}
yaql_raw_function!("replaceMany", list_replace_many, ArgSpec::Min(3), [Type::Array, Type::Int, Type::Array], false);

yaql_function!("indexOf", list_index_of(arr: Vec<Primitive>, item: Any) -> i64 {
    arr.iter().position(|e| crate::lang::primitive::primitive_eq(e, &item.0)).map(|p| p as i64).unwrap_or(-1)
});

yaql_function!("lastIndexOf", list_last_index_of(arr: Vec<Primitive>, item: Any) -> i64 {
    arr.iter().rposition(|e| crate::lang::primitive::primitive_eq(e, &item.0)).map(|p| p as i64).unwrap_or(-1)
});