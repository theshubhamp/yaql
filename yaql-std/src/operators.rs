use yaql_core::lang::Primitive;
use yaql_core::lang::FUNCTIONS;
use yaql_core::lang::functions::{EvalError, Number, Any, Null, SetVec, RegexWrapper};
use yaql_core::lang::functions::dispatch;
use yaql_macros::yaql_function;
use std::collections::HashMap;

pub fn dot_access_impl(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::Map(map), Primitive::String(key)) => map.get(key).cloned().unwrap_or(Primitive::Null),
        (Primitive::Array(arr), Primitive::String(_key)) => {
            Primitive::Array(arr.iter().map(|e| dot_access_impl(e.clone(), right.clone())).collect())
        }
        _ => Primitive::Null,
    }
}

pub fn add_primitives(acc: Primitive, e: Primitive) -> Result<Primitive, EvalError> {
    dispatch(
        FUNCTIONS.lookup("+"),
        vec![acc, e],
        vec![],
    )
}

#[yaql_function("+")]
fn add_str(l: String, r: String) -> String { format!("{}{}", l, r) }

#[yaql_function("+")]
fn add_arr(l: Vec<Primitive>, r: Vec<Primitive>) -> Vec<Primitive> { let mut c = l; c.extend(r); c }

#[yaql_function("+")]
fn add_set_set(l: SetVec, r: SetVec) -> SetVec {
    let mut c = l.0;
    for e in &r.0 { yaql_core::lang::sets::set_push_unique(&mut c, e); }
    SetVec(c)
}

#[yaql_function("+")]
fn add_set_arr(l: SetVec, r: Vec<Primitive>) -> Vec<Primitive> {
    let mut c = l.0; c.extend(r); c
}

#[yaql_function("+")]
fn add_arr_set(l: Vec<Primitive>, r: SetVec) -> Vec<Primitive> {
    let mut c = l; c.extend(r.0); c
}

#[yaql_function("+")]
fn add_map(l: HashMap<String, Primitive>, r: HashMap<String, Primitive>) -> HashMap<String, Primitive> {
    let mut c = l; for (k, v) in r { c.insert(k, v); } c
}

#[yaql_function("+")]
fn add_int(l: i64, r: i64) -> i64 { l + r }

#[yaql_function("+")]
fn add_num(l: Number, r: Number) -> f64 { l.0 + r.0 }

#[yaql_function("-")]
fn sub_set(l: SetVec, r: SetVec) -> SetVec {
    SetVec(yaql_core::lang::sets::set_difference(&l.0, &r.0))
}

#[yaql_function("-")]
fn sub_int(l: i64, r: i64) -> i64 { l - r }

#[yaql_function("-")]
fn sub_num(l: Number, r: Number) -> f64 { l.0 - r.0 }

#[yaql_function("*")]
fn mul_str_int(s: String, n: i64) -> String { s.repeat(n as usize) }

#[yaql_function("*")]
fn mul_int_str(n: i64, s: String) -> String { s.repeat(n as usize) }

#[yaql_function("*")]
fn mul_arr_int(a: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    (0..n).flat_map(|_| a.iter().cloned()).collect()
}

#[yaql_function("*")]
fn mul_int_arr(n: i64, a: Vec<Primitive>) -> Vec<Primitive> {
    (0..n).flat_map(|_| a.iter().cloned()).collect()
}

#[yaql_function("*")]
fn mul_int(l: i64, r: i64) -> i64 { l * r }

#[yaql_function("*")]
fn mul_num(l: Number, r: Number) -> f64 { l.0 * r.0 }

#[yaql_function("/")]
pub fn div_int(l: i64, r: i64) -> Result<i64, EvalError> {
    if r == 0 {
        return Err(EvalError::new("division by zero"));
    }
    Ok((l as f64 / r as f64).floor() as i64)
}
#[yaql_function("/")]
pub fn div_num(l: Number, r: Number) -> Result<f64, EvalError> {
    if r.0 == 0.0 {
        return Err(EvalError::new("division by zero"));
    }
    Ok(l.0 / r.0)
}
#[yaql_function("mod")]
fn mod_int(l: i64, r: i64) -> i64 {
    l - r * ((l as f64 / r as f64).floor() as i64)
}

#[yaql_function("mod")]
fn mod_num(l: Number, r: Number) -> f64 {
    l.0 - r.0 * (l.0 / r.0).floor()
}

#[yaql_function("=")]
fn eq_str(l: String, r: String) -> bool { l == r }

#[yaql_function("=")]
fn eq_num(l: Number, r: Number) -> bool { l.0 == r.0 }

#[yaql_function("=")]
fn eq_bool(l: bool, r: bool) -> bool { l == r }

#[yaql_function("=")]
fn eq_num_bool(l: Number, r: bool) -> bool { (l.0 == 1.0) == r }

#[yaql_function("=")]
fn eq_bool_num(l: bool, r: Number) -> bool { l == (r.0 == 1.0) }

#[yaql_function("=")]
fn eq_null(_l: Null, _r: Null) -> bool { true }

#[yaql_function("=")]
fn eq_arr(l: Vec<Primitive>, r: Vec<Primitive>) -> bool {
    l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| yaql_core::lang::primitive_eq(a, b))
}

#[yaql_function("=")]
fn eq_set(l: SetVec, r: SetVec) -> bool {
    yaql_core::lang::sets::set_equal(&l.0, &r.0)
}

#[yaql_function("=")]
fn eq_map(l: HashMap<String, Primitive>, r: HashMap<String, Primitive>) -> bool {
    l.len() == r.len() && l.iter().all(|(k, v)| r.get(k).is_some_and(|rv| yaql_core::lang::primitive_eq(v, rv)))
}

#[yaql_function("=")]
fn eq_any(_l: Any, _r: Any) -> bool { false }

#[yaql_function("!=")]
fn neq(l: Any, r: Any) -> bool { !yaql_core::lang::primitive_eq(&l.0, &r.0) }

#[yaql_function("<")]
fn lt_set(l: SetVec, r: SetVec) -> bool {
    l.0.len() < r.0.len() && yaql_core::lang::sets::is_subset(&l.0, &r.0)
}

#[yaql_function("<")]
fn lt_null_null(_l: Null, _r: Null) -> bool { false }

#[yaql_function("<")]
fn lt_null_any(_l: Null, _r: Any) -> bool { true }

#[yaql_function("<")]
fn lt_any_null(_l: Any, _r: Null) -> bool { false }

#[yaql_function("<")]
fn lt_num(l: Number, r: Number) -> bool { l.0 < r.0 }

#[yaql_function("<")]
fn lt_any(_l: Any, _r: Any) -> bool { false }

#[yaql_function("<=")]
fn lteq_set(l: SetVec, r: SetVec) -> bool {
    yaql_core::lang::sets::is_subset(&l.0, &r.0)
}

#[yaql_function("<=")]
fn lteq_null_null(_l: Null, _r: Null) -> bool { true }

#[yaql_function("<=")]
fn lteq_null_any(_l: Null, _r: Any) -> bool { true }

#[yaql_function("<=")]
fn lteq_any_null(_l: Any, _r: Null) -> bool { false }

#[yaql_function("<=")]
fn lteq_num(l: Number, r: Number) -> bool { l.0 <= r.0 }

#[yaql_function("<=")]
fn lteq_any(_l: Any, _r: Any) -> bool { false }

#[yaql_function(">")]
fn gt_set(l: SetVec, r: SetVec) -> bool {
    l.0.len() > r.0.len() && yaql_core::lang::sets::is_subset(&r.0, &l.0)
}

#[yaql_function(">")]
fn gt_null_null(_l: Null, _r: Null) -> bool { false }

#[yaql_function(">")]
fn gt_null_any(_l: Null, _r: Any) -> bool { false }

#[yaql_function(">")]
fn gt_any_null(_l: Any, _r: Null) -> bool { true }

#[yaql_function(">")]
fn gt_num(l: Number, r: Number) -> bool { l.0 > r.0 }

#[yaql_function(">")]
fn gt_any(_l: Any, _r: Any) -> bool { false }

#[yaql_function(">=")]
fn gteq_set(l: SetVec, r: SetVec) -> bool {
    yaql_core::lang::sets::is_subset(&r.0, &l.0)
}

#[yaql_function(">=")]
fn gteq_null_null(_l: Null, _r: Null) -> bool { true }

#[yaql_function(">=")]
fn gteq_null_any(_l: Null, _r: Any) -> bool { false }

#[yaql_function(">=")]
fn gteq_any_null(_l: Any, _r: Null) -> bool { true }

#[yaql_function(">=")]
fn gteq_num(l: Number, r: Number) -> bool { l.0 >= r.0 }

#[yaql_function(">=")]
fn gteq_any(_l: Any, _r: Any) -> bool { false }

#[yaql_function("in")]
fn in_str(l: String, r: String) -> bool { r.contains(l.as_str()) }

#[yaql_function("in")]
fn in_arr(l: Any, r: Vec<Primitive>) -> bool {
    r.iter().any(|e| yaql_core::lang::primitive_eq(e, &l.0))
}

#[yaql_function("in")]
fn in_set(l: Any, r: SetVec) -> bool {
    r.0.iter().any(|e| yaql_core::lang::primitive_eq(e, &l.0))
}

#[yaql_function("in")]
fn in_any(_l: Any, _r: Any) -> bool { false }

#[yaql_function(".")]
fn dot_map(l: HashMap<String, Primitive>, r: String) -> Primitive {
    l.get(&r).cloned().unwrap_or(Primitive::Null)
}

#[yaql_function(".")]
fn dot_arr(l: Vec<Primitive>, r: String) -> Primitive {
    crate::operators::dot_access_impl(Primitive::Array(l), Primitive::String(r))
}

#[yaql_function(".")]
fn dot_any(_l: Any, _r: Any) -> Primitive { Primitive::Null }

#[yaql_function("?.")]
fn dot_map_opt(l: HashMap<String, Primitive>, r: String) -> Primitive {
    l.get(&r).cloned().unwrap_or(Primitive::Null)
}

#[yaql_function("?.")]
fn dot_arr_opt(l: Vec<Primitive>, r: String) -> Primitive {
    crate::operators::dot_access_impl(Primitive::Array(l), Primitive::String(r))
}

#[yaql_function("?.")]
fn dot_any_opt(_l: Any, _r: Any) -> Primitive { Primitive::Null }

#[yaql_function("=>")]
fn mapping_arrow(_l: Any, _r: Any) -> Primitive { Primitive::Null }

#[yaql_function("=~")]
fn match_regex(s: String, re: RegexWrapper) -> bool {
    re.0.is_match(s.as_str())
}

#[yaql_function("=~")]
fn match_pattern(s: String, p: String) -> bool {
    let re = regex::Regex::new(p.as_str())
        .unwrap_or_else(|_| regex::Regex::new(&regex::escape(p.as_str())).unwrap());
    re.is_match(s.as_str())
}

#[yaql_function("=~")]
fn match_any(_l: Any, _r: Any) -> Primitive { Primitive::Null }

#[yaql_function("!~")]
fn nmatch_regex(s: String, re: RegexWrapper) -> bool {
    !re.0.is_match(s.as_str())
}

#[yaql_function("!~")]
fn nmatch_pattern(s: String, p: String) -> bool {
    let re = regex::Regex::new(p.as_str())
        .unwrap_or_else(|_| regex::Regex::new(&regex::escape(p.as_str())).unwrap());
    !re.is_match(s.as_str())
}

#[yaql_function("!~")]
fn nmatch_any(_l: Any, _r: Any) -> Primitive { Primitive::Null }
