use crate::lang::primitive::Primitive;
use crate::yaql_function;

// --- Internal helpers (used by other modules) ---

pub fn dot_access_impl(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::Map(map), Primitive::String(key)) => map.get(key).cloned().unwrap_or(Primitive::Null),
        (Primitive::Array(arr), Primitive::String(_key)) => {
            Primitive::Array(arr.iter().map(|e| dot_access_impl(e.clone(), right.clone())).collect())
        }
        _ => Primitive::Null,
    }
}

// Used by query.rs `sum` to fold array elements via the "+" semantics.
pub fn add_primitives(acc: Primitive, e: Primitive) -> Primitive {
    crate::interpreter::dispatch(
        crate::lang::FUNCTIONS.lookup("+"),
        vec![acc, e],
        vec![],
    )
}

// --- "+" ---
yaql_function!("+", add_str(l: String, r: String) -> String { format!("{}{}", l, r) });
yaql_function!("+", add_arr(l: Vec<Primitive>, r: Vec<Primitive>) -> Vec<Primitive> { let mut c = l; c.extend(r); c });
yaql_function!("+", add_set_set(l: SetVec, r: SetVec) -> SetVec {
    let mut c = l.0;
    for e in &r.0 { crate::lang::sets::set_push_unique(&mut c, e); }
    SetVec(c)
});
yaql_function!("+", add_set_arr(l: SetVec, r: Vec<Primitive>) -> Vec<Primitive> {
    let mut c = l.0; c.extend(r); c
});
yaql_function!("+", add_arr_set(l: Vec<Primitive>, r: SetVec) -> Vec<Primitive> {
    let mut c = l; c.extend(r.0); c
});
yaql_function!("+", add_map(l: HashMap<String, Primitive>, r: HashMap<String, Primitive>) -> HashMap<String, Primitive> {
    let mut c = l; for (k, v) in r { c.insert(k, v); } c
});
yaql_function!("+", add_int(l: i64, r: i64) -> i64 { l + r });
yaql_function!("+", add_num(l: Number, r: Number) -> f64 { l.0 + r.0 });

// --- "-" ---
yaql_function!("-", sub_set(l: SetVec, r: SetVec) -> SetVec {
    SetVec(crate::lang::sets::set_difference(&l.0, &r.0))
});
yaql_function!("-", sub_int(l: i64, r: i64) -> i64 { l - r });
yaql_function!("-", sub_num(l: Number, r: Number) -> f64 { l.0 - r.0 });

// --- "*" ---
yaql_function!("*", mul_str_int(s: String, n: i64) -> String { s.repeat(n as usize) });
yaql_function!("*", mul_int_str(n: i64, s: String) -> String { s.repeat(n as usize) });
yaql_function!("*", mul_arr_int(a: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    (0..n).flat_map(|_| a.iter().cloned()).collect()
});
yaql_function!("*", mul_int_arr(n: i64, a: Vec<Primitive>) -> Vec<Primitive> {
    (0..n).flat_map(|_| a.iter().cloned()).collect()
});
yaql_function!("*", mul_int(l: i64, r: i64) -> i64 { l * r });
yaql_function!("*", mul_num(l: Number, r: Number) -> f64 { l.0 * r.0 });

// --- "/" ---
yaql_function!("/", div_int(l: i64, r: i64) -> i64 {
    if r == 0 { panic!("division by zero") }
    (l as f64 / r as f64).floor() as i64
});
yaql_function!("/", div_num(l: Number, r: Number) -> f64 {
    if r.0 == 0.0 { panic!("division by zero") }
    l.0 / r.0
});

// --- "mod" ---
yaql_function!("mod", mod_int(l: i64, r: i64) -> i64 {
    l - r * ((l as f64 / r as f64).floor() as i64)
});
yaql_function!("mod", mod_num(l: Number, r: Number) -> f64 {
    l.0 - r.0 * (l.0 / r.0).floor()
});

// --- "=" ---
yaql_function!("=", eq_str(l: String, r: String) -> bool { l == r });
yaql_function!("=", eq_num(l: Number, r: Number) -> bool { l.0 == r.0 });
yaql_function!("=", eq_bool(l: bool, r: bool) -> bool { l == r });
yaql_function!("=", eq_num_bool(l: Number, r: bool) -> bool { (l.0 == 1.0) == r });
yaql_function!("=", eq_bool_num(l: bool, r: Number) -> bool { l == (r.0 == 1.0) });
yaql_function!("=", eq_null(l: Null, r: Null) -> bool { true });
yaql_function!("=", eq_arr(l: Vec<Primitive>, r: Vec<Primitive>) -> bool {
    l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| crate::lang::primitive::primitive_eq(a, b))
});
yaql_function!("=", eq_set(l: SetVec, r: SetVec) -> bool {
    crate::lang::sets::set_equal(&l.0, &r.0)
});
yaql_function!("=", eq_map(l: HashMap<String, Primitive>, r: HashMap<String, Primitive>) -> bool {
    l.len() == r.len() && l.iter().all(|(k, v)| r.get(k).map_or(false, |rv| crate::lang::primitive::primitive_eq(v, rv)))
});
yaql_function!("=", eq_any(l: Any, r: Any) -> bool { false });

// --- "!=" ---
yaql_function!("!=", neq(l: Any, r: Any) -> bool { !crate::lang::primitive::primitive_eq(&l.0, &r.0) });

// --- "<" ---
yaql_function!("<", lt_set(l: SetVec, r: SetVec) -> bool {
    l.0.len() < r.0.len() && crate::lang::sets::is_subset(&l.0, &r.0)
});
yaql_function!("<", lt_null_null(l: Null, r: Null) -> bool { false });
yaql_function!("<", lt_null_any(l: Null, r: Any) -> bool { true });
yaql_function!("<", lt_any_null(l: Any, r: Null) -> bool { false });
yaql_function!("<", lt_num(l: Number, r: Number) -> bool { l.0 < r.0 });
yaql_function!("<", lt_any(l: Any, r: Any) -> bool { false });

// --- "<=" ---
yaql_function!("<=", lteq_set(l: SetVec, r: SetVec) -> bool {
    crate::lang::sets::is_subset(&l.0, &r.0)
});
yaql_function!("<=", lteq_null_null(l: Null, r: Null) -> bool { true });
yaql_function!("<=", lteq_null_any(l: Null, r: Any) -> bool { true });
yaql_function!("<=", lteq_any_null(l: Any, r: Null) -> bool { false });
yaql_function!("<=", lteq_num(l: Number, r: Number) -> bool { l.0 <= r.0 });
yaql_function!("<=", lteq_any(l: Any, r: Any) -> bool { false });

// --- ">" ---
yaql_function!(">", gt_set(l: SetVec, r: SetVec) -> bool {
    l.0.len() > r.0.len() && crate::lang::sets::is_subset(&r.0, &l.0)
});
yaql_function!(">", gt_null_null(l: Null, r: Null) -> bool { false });
yaql_function!(">", gt_null_any(l: Null, r: Any) -> bool { false });
yaql_function!(">", gt_any_null(l: Any, r: Null) -> bool { true });
yaql_function!(">", gt_num(l: Number, r: Number) -> bool { l.0 > r.0 });
yaql_function!(">", gt_any(l: Any, r: Any) -> bool { false });

// --- ">=" ---
yaql_function!(">=", gteq_set(l: SetVec, r: SetVec) -> bool {
    crate::lang::sets::is_subset(&r.0, &l.0)
});
yaql_function!(">=", gteq_null_null(l: Null, r: Null) -> bool { true });
yaql_function!(">=", gteq_null_any(l: Null, r: Any) -> bool { false });
yaql_function!(">=", gteq_any_null(l: Any, r: Null) -> bool { true });
yaql_function!(">=", gteq_num(l: Number, r: Number) -> bool { l.0 >= r.0 });
yaql_function!(">=", gteq_any(l: Any, r: Any) -> bool { false });

// --- "in" ---
yaql_function!("in", in_str(l: String, r: String) -> bool { r.contains(l.as_str()) });
yaql_function!("in", in_arr(l: Any, r: Vec<Primitive>) -> bool {
    r.iter().any(|e| crate::lang::primitive::primitive_eq(e, &l.0))
});
yaql_function!("in", in_set(l: Any, r: SetVec) -> bool {
    r.0.iter().any(|e| crate::lang::primitive::primitive_eq(e, &l.0))
});
yaql_function!("in", in_any(l: Any, r: Any) -> bool { false });

// --- "." ---
yaql_function!(".", dot_map(l: HashMap<String, Primitive>, r: String) -> Primitive {
    l.get(&r).cloned().unwrap_or(Primitive::Null)
});
yaql_function!(".", dot_arr(l: Vec<Primitive>, r: String) -> Primitive {
    crate::lang::operators::dot_access_impl(Primitive::Array(l), Primitive::String(r))
});
yaql_function!(".", dot_any(l: Any, r: Any) -> Primitive { Primitive::Null });

// --- "?." (same as "." — interpreter handles the Null check before dispatch) ---
yaql_function!("?.", dot_map_opt(l: HashMap<String, Primitive>, r: String) -> Primitive {
    l.get(&r).cloned().unwrap_or(Primitive::Null)
});
yaql_function!("?.", dot_arr_opt(l: Vec<Primitive>, r: String) -> Primitive {
    crate::lang::operators::dot_access_impl(Primitive::Array(l), Primitive::String(r))
});
yaql_function!("?.", dot_any_opt(l: Any, r: Any) -> Primitive { Primitive::Null });

// --- "=>" ---
yaql_function!("=>", mapping_arrow(l: Any, r: Any) -> Primitive { Primitive::Null });

// --- "=~" ---
yaql_function!("=~", match_regex(s: String, re: RegexWrapper) -> bool {
    re.0.is_match(s.as_str())
});
yaql_function!("=~", match_pattern(s: String, p: String) -> bool {
    let re = regex::Regex::new(p.as_str())
        .unwrap_or_else(|_| regex::Regex::new(&regex::escape(p.as_str())).unwrap());
    re.is_match(s.as_str())
});
yaql_function!("=~", match_any(l: Any, r: Any) -> Primitive { Primitive::Null });

// --- "!~" ---
yaql_function!("!~", nmatch_regex(s: String, re: RegexWrapper) -> bool {
    !re.0.is_match(s.as_str())
});
yaql_function!("!~", nmatch_pattern(s: String, p: String) -> bool {
    let re = regex::Regex::new(p.as_str())
        .unwrap_or_else(|_| regex::Regex::new(&regex::escape(p.as_str())).unwrap());
    !re.is_match(s.as_str())
});
yaql_function!("!~", nmatch_any(l: Any, r: Any) -> Primitive { Primitive::Null });