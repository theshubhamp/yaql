use crate::lang::primitive::{Primitive, compare};
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::{ArgSpec, Type};
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

yaql_function!("deleteAll", dict_delete_all_fn(m: HashMap<String, Primitive>, keys: Vec<Primitive>) -> HashMap<String, Primitive> {
    let mut m = m;
    for k in &keys {
        let key = match k {
            Primitive::String(s) => s.clone(),
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => continue,
        };
        m.remove(&key);
    }
    m
});

yaql_function!("contains", contains_array(arr: Vec<Primitive>, item: Any) -> bool {
    arr.iter().any(|e| crate::lang::primitive::primitive_eq(e, &item.0))
});

yaql_function!("contains", contains_string(s: String, sub: String) -> bool {
    s.contains(sub.as_str())
});

pub fn max_impl(iter: Vec<Primitive>) -> Primitive {
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

pub fn max_varargs(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    max_impl(args)
}
yaql_raw_function!("max", max_varargs, ArgSpec::Min(1), [Type::Any], false);

yaql_function!("max", max_arr(arr: Vec<Primitive>) -> Primitive { crate::lang::collections::max_impl(arr) });
yaql_function!("max", max_set(arr: SetVec) -> Primitive { crate::lang::collections::max_impl(arr.0) });
yaql_function!("max", max_arr_default(arr: Vec<Primitive>, default: Any) -> Primitive {
    if arr.is_empty() { return default.0 }
    crate::lang::collections::max_impl(arr)
});
yaql_function!("max", max_set_default(arr: SetVec, default: Any) -> Primitive {
    if arr.0.is_empty() { return default.0 }
    crate::lang::collections::max_impl(arr.0)
});

pub fn min_impl(iter: Vec<Primitive>) -> Primitive {
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

pub fn min_varargs(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    min_impl(args)
}
yaql_raw_function!("min", min_varargs, ArgSpec::Min(1), [Type::Any], false);

yaql_function!("min", min_arr(arr: Vec<Primitive>) -> Primitive { crate::lang::collections::min_impl(arr) });
yaql_function!("min", min_set(arr: SetVec) -> Primitive { crate::lang::collections::min_impl(arr.0) });
yaql_function!("min", min_arr_default(arr: Vec<Primitive>, default: Any) -> Primitive {
    if arr.is_empty() { return default.0 }
    crate::lang::collections::min_impl(arr)
});
yaql_function!("min", min_set_default(arr: SetVec, default: Any) -> Primitive {
    if arr.0.is_empty() { return default.0 }
    crate::lang::collections::min_impl(arr.0)
});

// --- List mutation ---

pub(crate) fn norm_idx(i: i64, len: usize) -> usize {
    if i < 0 { ((len as i64) + i).max(0) as usize } else { (i as usize).min(len) }
}

yaql_function!("delete", list_delete_2(arr: Vec<Primitive>, index: i64) -> Vec<Primitive> {
    let start = crate::lang::collections::norm_idx(index, arr.len());
    let mut result = arr;
    let end = (start + 1).min(result.len());
    result.drain(start..end);
    result
});

yaql_function!("delete", list_delete_3(arr: Vec<Primitive>, index: i64, count: i64) -> Vec<Primitive> {
    let start = crate::lang::collections::norm_idx(index, arr.len());
    let count = if count < 0 { arr.len() - start } else { count as usize };
    let mut result = arr;
    let end = (start + count).min(result.len());
    result.drain(start..end);
    result
});

yaql_function!("insert", list_insert(arr: Vec<Primitive>, pos: i64, value: Any) -> Vec<Primitive> {
    let mut pos = crate::lang::collections::norm_idx(pos, arr.len());
    pos = pos.min(arr.len());
    let mut result = arr;
    result.insert(pos, value.0);
    result
});

yaql_function!("insertMany", list_insert_many(arr: Vec<Primitive>, pos: i64, items: Vec<Primitive>) -> Vec<Primitive> {
    let pos = if pos < 0 { 0 } else { (pos as usize).min(arr.len()) };
    let mut result = arr;
    for (i, item) in items.iter().enumerate() {
        result.insert(pos + i, item.clone());
    }
    result
});

yaql_function!("replace", list_replace_3(arr: Vec<Primitive>, index: i64, value: Any) -> Vec<Primitive> {
    let start = crate::lang::collections::norm_idx(index, arr.len());
    let mut result = arr;
    if start < result.len() { result[start] = value.0; }
    result
});

yaql_function!("replace", list_replace_4(arr: Vec<Primitive>, index: i64, value: Any, count: i64) -> Vec<Primitive> {
    let start = crate::lang::collections::norm_idx(index, arr.len());
    let count = if count < 0 { arr.len() - start } else { count as usize };
    let mut result = arr;
    let end = (start + count).min(result.len());
    if count == 1 {
        if start < result.len() { result[start] = value.0; }
    } else {
        result.drain(start..end);
        result.insert(start, value.0);
    }
    result
});

yaql_function!("replaceMany", list_replace_many_3(arr: Vec<Primitive>, index: i64, items: Vec<Primitive>) -> Vec<Primitive> {
    let start = crate::lang::collections::norm_idx(index, arr.len());
    let mut result = arr;
    let end = (start + 1).min(result.len());
    result.drain(start..end);
    for (i, item) in items.iter().enumerate() {
        result.insert(start + i, item.clone());
    }
    result
});

yaql_function!("replaceMany", list_replace_many_4(arr: Vec<Primitive>, index: i64, items: Vec<Primitive>, count: i64) -> Vec<Primitive> {
    let start = crate::lang::collections::norm_idx(index, arr.len());
    let count = if count < 0 { arr.len() - start } else { count as usize };
    let mut result = arr;
    let end = (start + count).min(result.len());
    result.drain(start..end);
    for (i, item) in items.iter().enumerate() {
        result.insert(start + i, item.clone());
    }
    result
});

yaql_function!("indexOf", list_index_of(arr: Vec<Primitive>, item: Any) -> i64 {
    arr.iter().position(|e| crate::lang::primitive::primitive_eq(e, &item.0)).map(|p| p as i64).unwrap_or(-1)
});

yaql_function!("lastIndexOf", list_last_index_of(arr: Vec<Primitive>, item: Any) -> i64 {
    arr.iter().rposition(|e| crate::lang::primitive::primitive_eq(e, &item.0)).map(|p| p as i64).unwrap_or(-1)
});