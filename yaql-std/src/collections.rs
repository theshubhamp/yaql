use yaql_core::lang::{Primitive, compare};
use yaql_core::lang::functions::{EvalError, Varargs, Kwargs, Any, SetVec};
use yaql_macros::yaql_function;
use std::collections::HashMap;

#[yaql_function("get")]
fn get_fn(m: HashMap<String, Primitive>, key: String) -> Primitive {
    m.get(&key).cloned().unwrap_or(Primitive::Null)
}

#[yaql_function("get")]
fn get_default(m: HashMap<String, Primitive>, key: String, default: Any) -> Primitive {
    m.get(&key).cloned().unwrap_or(default.0)
}

#[yaql_function("keys")]
fn keys_fn(m: HashMap<String, Primitive>) -> Vec<Primitive> {
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    keys.into_iter().map(Primitive::String).collect()
}

#[yaql_function("values")]
fn values_fn(m: HashMap<String, Primitive>) -> Vec<Primitive> {
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    keys.into_iter().filter_map(|k| m.get(&k).cloned()).collect()
}

#[yaql_function("items")]
fn items_fn(m: HashMap<String, Primitive>) -> Vec<Primitive> {
    let mut keys: Vec<String> = m.keys().cloned().collect();
    keys.sort();
    keys.into_iter().map(|k| {
        let v = m.get(&k).cloned().unwrap_or(Primitive::Null);
        Primitive::Array(vec![Primitive::String(k), v])
    }).collect()
}

#[yaql_function("containsKey")]
fn contains_key_str(m: HashMap<String, Primitive>, key: String) -> bool {
    m.contains_key(&key)
}

#[yaql_function("containsKey")]
fn contains_key_any(m: HashMap<String, Primitive>, _key: Any) -> bool {
    false
}

#[yaql_function("list")]
pub fn list_fn(args: Varargs<0>) -> Vec<Primitive> {
    args.0
}
#[yaql_function("toList")]
fn to_list_fn(a: Vec<Primitive>) -> Vec<Primitive> { a }

#[yaql_function("dict")]
pub fn dict_fn(args: Varargs<0>, kwargs: Kwargs) -> Result<Primitive, EvalError> {
    let mut map = HashMap::new();
    let pairs: Vec<Primitive> = if args.0.len() == 1 {
        if let Primitive::Array(a) = &args.0[0] { a.clone() } else { args.0 }
    } else { args.0 };
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
    for (k, v) in kwargs.0 {
        let key = match k {
            Primitive::String(s) => s,
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => continue,
        };
        map.insert(key, v);
    }
    Ok(Primitive::Map(map))
}
#[yaql_function("set")]
pub fn dict_set_fn(m: HashMap<String, Primitive>, rest: Varargs<0>, kwargs: Kwargs) -> Result<Primitive, EvalError> {
    if rest.0.is_empty() && kwargs.0.is_empty() {
        return Ok(Primitive::Set(crate::sets::set_fn(Varargs::<0>(vec![Primitive::Map(m)])).0));
    }
    let mut m = m;
    if rest.0.len() == 2 {
        let key = match &rest.0[0] {
            Primitive::String(s) => s.clone(),
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => return Ok(Primitive::Null),
        };
        m.insert(key, rest.0[1].clone());
    } else if rest.0.len() == 1 {
        if let Primitive::Map(other) = &rest.0[0] {
            for (k, v) in other {
                m.insert(k.clone(), v.clone());
            }
        }
    }
    for (k, v) in kwargs.0 {
        let key = match k {
            Primitive::String(s) => s,
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => continue,
        };
        m.insert(key, v);
    }
    Ok(Primitive::Map(m))
}
#[yaql_function("delete")]
pub fn dict_delete_fn(m: HashMap<String, Primitive>, rest: Varargs<1>) -> HashMap<String, Primitive> {
    let mut m = m;
    for arg in &rest.0 {
        let key = match arg {
            Primitive::String(s) => s.clone(),
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => continue,
        };
        m.remove(&key);
    }
    m
}
#[yaql_function("deleteAll")]
fn dict_delete_all_fn(m: HashMap<String, Primitive>, keys: Vec<Primitive>) -> HashMap<String, Primitive> {
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
}

#[yaql_function("contains")]
fn contains_array(arr: Vec<Primitive>, item: Any) -> bool {
    arr.iter().any(|e| yaql_core::lang::primitive_eq(e, &item.0))
}

#[yaql_function("contains")]
fn contains_string(s: String, sub: String) -> bool {
    s.contains(sub.as_str())
}

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

#[yaql_function("max")]
pub fn max_varargs(args: Varargs<1>) -> Primitive {
    max_impl(args.0)
}
#[yaql_function("max")]
fn max_arr(arr: Vec<Primitive>) -> Primitive { crate::collections::max_impl(arr) }

#[yaql_function("max")]
fn max_set(arr: SetVec) -> Primitive { crate::collections::max_impl(arr.0) }

#[yaql_function("max")]
fn max_arr_default(arr: Vec<Primitive>, default: Any) -> Primitive {
    if arr.is_empty() { return default.0 }
    crate::collections::max_impl(arr)
}

#[yaql_function("max")]
fn max_set_default(arr: SetVec, default: Any) -> Primitive {
    if arr.0.is_empty() { return default.0 }
    crate::collections::max_impl(arr.0)
}

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

#[yaql_function("min")]
pub fn min_varargs(args: Varargs<1>) -> Primitive {
    min_impl(args.0)
}
#[yaql_function("min")]
fn min_arr(arr: Vec<Primitive>) -> Primitive { crate::collections::min_impl(arr) }

#[yaql_function("min")]
fn min_set(arr: SetVec) -> Primitive { crate::collections::min_impl(arr.0) }

#[yaql_function("min")]
fn min_arr_default(arr: Vec<Primitive>, default: Any) -> Primitive {
    if arr.is_empty() { return default.0 }
    crate::collections::min_impl(arr)
}

#[yaql_function("min")]
fn min_set_default(arr: SetVec, default: Any) -> Primitive {
    if arr.0.is_empty() { return default.0 }
    crate::collections::min_impl(arr.0)
}

pub(crate) fn norm_idx(i: i64, len: usize) -> usize {
    if i < 0 { ((len as i64) + i).max(0) as usize } else { (i as usize).min(len) }
}

#[yaql_function("delete")]
fn list_delete_2(arr: Vec<Primitive>, index: i64) -> Vec<Primitive> {
    let start = crate::collections::norm_idx(index, arr.len());
    let mut result = arr;
    let end = (start + 1).min(result.len());
    result.drain(start..end);
    result
}

#[yaql_function("delete")]
fn list_delete_3(arr: Vec<Primitive>, index: i64, count: i64) -> Vec<Primitive> {
    let start = crate::collections::norm_idx(index, arr.len());
    let count = if count < 0 { arr.len() - start } else { count as usize };
    let mut result = arr;
    let end = (start + count).min(result.len());
    result.drain(start..end);
    result
}

#[yaql_function("insert")]
fn list_insert(arr: Vec<Primitive>, pos: i64, value: Any) -> Vec<Primitive> {
    let mut pos = crate::collections::norm_idx(pos, arr.len());
    pos = pos.min(arr.len());
    let mut result = arr;
    result.insert(pos, value.0);
    result
}

#[yaql_function("insertMany")]
fn list_insert_many(arr: Vec<Primitive>, pos: i64, items: Vec<Primitive>) -> Vec<Primitive> {
    let pos = if pos < 0 { 0 } else { (pos as usize).min(arr.len()) };
    let mut result = arr;
    for (i, item) in items.iter().enumerate() {
        result.insert(pos + i, item.clone());
    }
    result
}

#[yaql_function("replace")]
fn list_replace_3(arr: Vec<Primitive>, index: i64, value: Any) -> Vec<Primitive> {
    let start = crate::collections::norm_idx(index, arr.len());
    let mut result = arr;
    if start < result.len() { result[start] = value.0; }
    result
}

#[yaql_function("replace")]
fn list_replace_4(arr: Vec<Primitive>, index: i64, value: Any, count: i64) -> Vec<Primitive> {
    let start = crate::collections::norm_idx(index, arr.len());
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
}

#[yaql_function("replaceMany")]
fn list_replace_many_3(arr: Vec<Primitive>, index: i64, items: Vec<Primitive>) -> Vec<Primitive> {
    let start = crate::collections::norm_idx(index, arr.len());
    let mut result = arr;
    let end = (start + 1).min(result.len());
    result.drain(start..end);
    for (i, item) in items.iter().enumerate() {
        result.insert(start + i, item.clone());
    }
    result
}

#[yaql_function("replaceMany")]
fn list_replace_many_4(arr: Vec<Primitive>, index: i64, items: Vec<Primitive>, count: i64) -> Vec<Primitive> {
    let start = crate::collections::norm_idx(index, arr.len());
    let count = if count < 0 { arr.len() - start } else { count as usize };
    let mut result = arr;
    let end = (start + count).min(result.len());
    result.drain(start..end);
    for (i, item) in items.iter().enumerate() {
        result.insert(start + i, item.clone());
    }
    result
}

#[yaql_function("indexOf")]
fn list_index_of(arr: Vec<Primitive>, item: Any) -> i64 {
    arr.iter().position(|e| yaql_core::lang::primitive_eq(e, &item.0)).map(|p| p as i64).unwrap_or(-1)
}

#[yaql_function("lastIndexOf")]
fn list_last_index_of(arr: Vec<Primitive>, item: Any) -> i64 {
    arr.iter().rposition(|e| yaql_core::lang::primitive_eq(e, &item.0)).map(|p| p as i64).unwrap_or(-1)
}
