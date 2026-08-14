use yaql_core::lang::Primitive;
use yaql_core::lang::primitive::LambdaBody;
use yaql_macros::yaql_function;
use yaql_macros::yaql_raw_function;
use yaql_core::lang::functions::ArgSpec;
use yaql_core::lang::functions::Type;
use yaql_core::interpreter::eval_lambda;
use std::cell::RefCell;

// Thread-local storage for sort keys (for thenBy/thenByDescending)
// Each element's sort key is a Vec<Primitive> (multiple sort keys compound)
// SORT_DESC tracks whether each key position was sorted descending
thread_local! {
    static SORT_KEYS: RefCell<Vec<Vec<Primitive>>> = RefCell::new(Vec::new());
    static SORT_DESC: RefCell<Vec<bool>> = RefCell::new(Vec::new());
}

pub fn store_sort_keys(keys: Vec<Vec<Primitive>>, desc: Vec<bool>) {
    SORT_KEYS.with(|sk| { *sk.borrow_mut() = keys; });
    SORT_DESC.with(|sd| { *sd.borrow_mut() = desc; });
}

pub fn load_sort_keys() -> (Vec<Vec<Primitive>>, Vec<bool>) {
    let keys = SORT_KEYS.with(|sk| sk.borrow().clone());
    let desc = SORT_DESC.with(|sd| sd.borrow().clone());
    (keys, desc)
}

#[yaql_function("skip")]
fn skip(arr: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    let n = (n as usize).min(arr.len());
    arr[n..].to_vec()
}

#[yaql_function("skip")]
fn skip_set(arr: SetVec, n: i64) -> SetVec {
    let n = (n as usize).min(arr.0.len());
    SetVec(arr.0[n..].to_vec())
}

#[yaql_function("take")]
fn take(arr: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    let n = (n as usize).min(arr.len());
    arr[..n].to_vec()
}

#[yaql_function("take")]
fn take_set(arr: SetVec, n: i64) -> SetVec {
    let n = (n as usize).min(arr.0.len());
    SetVec(arr.0[..n].to_vec())
}

#[yaql_function("limit")]
fn limit(arr: Vec<Primitive>, n: i64) -> Vec<Primitive> {
    let n = (n as usize).min(arr.len());
    arr[..n].to_vec()
}

#[yaql_function("count")]
fn count_array(arr: Vec<Primitive>) -> i64 { arr.len() as i64 }

#[yaql_function("count")]
fn count_set(arr: SetVec) -> i64 { arr.0.len() as i64 }

#[yaql_function("count")]
fn count_map(m: std::collections::HashMap<String, Primitive>) -> i64 { m.len() as i64 }

#[yaql_function("count")]
fn count_string(s: String) -> i64 { s.chars().count() as i64 }

#[yaql_function("first")]
fn first(arr: Vec<Primitive>) -> Option<Primitive> { arr.first().cloned() }

#[yaql_function("first")]
fn first_set(arr: SetVec) -> Option<Primitive> { arr.0.first().cloned() }

#[yaql_function("last")]
fn last(arr: Vec<Primitive>) -> Option<Primitive> { arr.last().cloned() }

#[yaql_function("last")]
fn last_set(arr: SetVec) -> Option<Primitive> { arr.0.last().cloned() }

#[yaql_function("first")]
fn first_default(arr: Vec<Primitive>, default: Any) -> Primitive {
    arr.first().cloned().unwrap_or(default.0)
}

#[yaql_function("first")]
fn first_default_set(arr: SetVec, default: Any) -> Primitive {
    arr.0.first().cloned().unwrap_or(default.0)
}

#[yaql_function("last")]
fn last_default(arr: Vec<Primitive>, default: Any) -> Primitive {
    arr.last().cloned().unwrap_or(default.0)
}

#[yaql_function("last")]
fn last_default_set(arr: SetVec, default: Any) -> Primitive {
    arr.0.last().cloned().unwrap_or(default.0)
}

#[yaql_function("range")]
fn range_one(n: i64) -> Vec<Primitive> { (0..n).map(Primitive::Int).collect() }

#[yaql_function("range")]
fn range_two(start: i64, end: i64) -> Vec<Primitive> { (start..end).map(Primitive::Int).collect() }

#[yaql_function("range")]
fn range_three(start: i64, end: i64, step: i64) -> Option<Vec<Primitive>> {
    if step == 0 {
        None
    } else if step > 0 {
        Some((0..).map(|i| start + i * step).take_while(|&x| x < end).map(Primitive::Int).collect())
    } else {
        Some((0..).map(|i| start + i * step).take_while(|&x| x > end).map(Primitive::Int).collect())
    }
}

#[yaql_raw_function("append", ArgSpec::Min(1), [Type::Array], false)]
pub fn append(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Some(Primitive::Array(arr)) = args.first() else { return Ok(Primitive::Null) };
    let mut result = arr.clone();
    result.extend(args[1..].iter().cloned());
    Ok(Primitive::Array(result))
}
#[yaql_function("reverse")]
fn reverse_array(arr: Vec<Primitive>) -> Vec<Primitive> {
    let mut r = arr; r.reverse(); r
}

#[yaql_function("reverse")]
fn reverse_set(arr: SetVec) -> SetVec {
    let mut r = arr.0; r.reverse(); SetVec(r)
}

#[yaql_function("reverse")]
fn reverse_string(s: String) -> String { s.chars().rev().collect() }

#[yaql_function("isIterable")]
fn is_iterable(v: Any) -> bool {
    matches!(v.0, Primitive::Array(_) | Primitive::Set(_))
}

#[yaql_function("distinct")]
fn distinct(arr: Vec<Primitive>) -> Vec<Primitive> {
    let mut seen = Vec::new();
    for e in &arr {
        yaql_core::lang::sets::set_push_unique(&mut seen, e);
    }
    seen
}

// sum: array.sum() or array.sum(init)
// Set dispatch (SetVec) reuses the same impl.
pub fn sum_impl(arr: Vec<Primitive>, init: Option<Primitive>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let init = if let Some(i) = init {
        i
    } else if arr.first().map(|e| matches!(e, Primitive::String(_))).unwrap_or(false) {
        Primitive::String(String::new())
    } else if arr.first().map(|e| matches!(e, Primitive::Array(_))).unwrap_or(false) {
        Primitive::Array(Vec::new())
    } else {
        Primitive::Int(0)
    };
    arr.iter().try_fold(init, |acc, e| crate::operators::add_primitives(acc, e.clone()))
}
#[yaql_function("sum")]
fn sum_arr(arr: Vec<Primitive>) -> Primitive { crate::query::sum_impl(arr, None).unwrap_or(Primitive::Null) }

#[yaql_function("sum")]
fn sum_set(arr: SetVec) -> Primitive { crate::query::sum_impl(arr.0, None).unwrap_or(Primitive::Null) }

#[yaql_function("sum")]
fn sum_arr_init(arr: Vec<Primitive>, init: Any) -> Primitive { crate::query::sum_impl(arr, Some(init.0)).unwrap_or(Primitive::Null) }

#[yaql_function("sum")]
fn sum_set_init(arr: SetVec, init: Any) -> Primitive { crate::query::sum_impl(arr.0, Some(init.0)).unwrap_or(Primitive::Null) }

#[yaql_function("splitAt")]
fn split_at_fn(arr: Vec<Primitive>, pos: i64) -> Vec<Primitive> {
    let len = arr.len() as i64;
    let pos = if pos < 0 { ((len + pos).max(0)) as usize } else { (pos as usize).min(arr.len()) };
    vec![
        Primitive::Array(arr[..pos].to_vec()),
        Primitive::Array(arr[pos..].to_vec()),
    ]
}

// --- enumerate ---
#[yaql_function("enumerate")]
fn enumerate_1(arr: Vec<Primitive>) -> Vec<Primitive> {
    arr.iter().enumerate().map(|(i, v)| {
        Primitive::Array(vec![Primitive::Int(i as i64), v.clone()])
    }).collect()
}

#[yaql_function("enumerate")]
fn enumerate_2(arr: Vec<Primitive>, start: i64) -> Vec<Primitive> {
    arr.iter().enumerate().map(|(i, v)| {
        Primitive::Array(vec![Primitive::Int(start + i as i64), v.clone()])
    }).collect()
}

// --- single ---
#[yaql_function("single")]
fn single_fn(arr: Vec<Primitive>) -> Primitive {
    if arr.len() == 1 { arr[0].clone() } else { Primitive::Null }
}

// --- slice (chunk into sublists of given size) ---
#[yaql_function("slice")]
fn slice_fn(arr: Vec<Primitive>, size: i64) -> Vec<Primitive> {
    let size = (size as usize).max(1);
    arr.chunks(size).map(|chunk| Primitive::Array(chunk.to_vec())).collect()
}

// --- any (no predicate: non-empty) ---
#[yaql_function("any")]
fn any_fn(arr: Vec<Primitive>) -> bool {
    !arr.is_empty()
}

// --- all (no predicate: all truthy) ---
#[yaql_function("all")]
fn all_fn(arr: Vec<Primitive>) -> bool {
    arr.iter().all(yaql_core::lang::truthy)
}

// --- defaultIfEmpty ---
#[yaql_function("defaultIfEmpty")]
fn default_if_empty_fn(arr: Vec<Primitive>, default: Any) -> Primitive {
    if arr.is_empty() { default.0 } else { Primitive::Array(arr) }
}

// --- Lambda-consuming functions ---

fn get_lambda(args: &[Primitive], idx: usize) -> Option<&LambdaBody> {
    if let Some(Primitive::Lambda(l)) = args.get(idx) { Some(l) } else { None }
}

// where: array.where(predicate) -> filtered array
#[yaql_raw_function("where", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut result = Vec::new();
    for e in arr {
        if yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?) {
            result.push(e.clone());
        }
    }
    Ok(Primitive::Array(result))
}
// select: array.select(selector) -> mapped array
#[yaql_raw_function("select", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn select_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut result = Vec::new();
    for e in arr {
        result.push(eval_lambda(lambda, e.clone())?);
    }
    Ok(Primitive::Array(result))
}
// selectMany: array.selectMany(selector) -> flatMap
#[yaql_raw_function("selectMany", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn select_many_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut result = Vec::new();
    for e in arr {
        match eval_lambda(lambda, e.clone())? {
            Primitive::Array(sub) => result.extend(sub),
            other => result.push(other),
        }
    }
    Ok(Primitive::Array(result))
}
// selectMany with a constant (non-lambda) arg -> flatten the constant for each element
#[yaql_function("selectMany")]
fn select_many_constant(arr: Vec<Primitive>, constant: Any) -> Vec<Primitive> {
    let mut result = Vec::new();
    for _ in arr {
        match &constant.0 {
            Primitive::Array(sub) => result.extend(sub.iter().cloned()),
            other => result.push(other.clone()),
        }
    }
    result
}

// orderBy: array.orderBy(selector) -> sorted array (stores sort keys for thenBy)
#[yaql_raw_function("orderBy", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn order_by_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut keyed: Vec<(Primitive, Primitive)> = Vec::new();
    for e in arr {
        keyed.push((eval_lambda(lambda, e.clone())?, e.clone()));
    }
    let mut indices: Vec<usize> = (0..keyed.len()).collect();
    indices.sort_by(|&a, &b| yaql_core::lang::compare(&keyed[a].0, &keyed[b].0));
    let sorted: Vec<Primitive> = indices.iter().map(|&i| keyed[i].1.clone()).collect();
    let sort_keys: Vec<Vec<Primitive>> = indices.iter().map(|&i| vec![keyed[i].0.clone()]).collect();
    crate::query::store_sort_keys(sort_keys, vec![false]);
    Ok(Primitive::Array(sorted))
}
// orderBy without selector -> sort directly (key = element itself)
#[yaql_function("orderBy")]
fn order_by_identity(arr: Vec<Primitive>, _any: Any) -> Vec<Primitive> {
    let mut sorted = arr;
    sorted.sort_by(|a, b| yaql_core::lang::compare(a, b));
    let keys: Vec<Vec<Primitive>> = sorted.iter().map(|e| vec![e.clone()]).collect();
    crate::query::store_sort_keys(keys, vec![false]);
    sorted
}

// orderByDescending
#[yaql_raw_function("orderByDescending", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn order_by_desc_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut keyed: Vec<(Primitive, Primitive)> = Vec::new();
    for e in arr {
        keyed.push((eval_lambda(lambda, e.clone())?, e.clone()));
    }
    let mut indices: Vec<usize> = (0..keyed.len()).collect();
    indices.sort_by(|&a, &b| yaql_core::lang::compare(&keyed[b].0, &keyed[a].0));
    let sorted: Vec<Primitive> = indices.iter().map(|&i| keyed[i].1.clone()).collect();
    let sort_keys: Vec<Vec<Primitive>> = indices.iter().map(|&i| vec![keyed[i].0.clone()]).collect();
    crate::query::store_sort_keys(sort_keys, vec![true]);
    Ok(Primitive::Array(sorted))
}
// orderByDescending without selector -> sort directly descending
#[yaql_function("orderByDescending")]
fn order_by_desc_identity(arr: Vec<Primitive>, _any: Any) -> Vec<Primitive> {
    let mut sorted = arr;
    sorted.sort_by(|a, b| yaql_core::lang::compare(b, a));
    let keys: Vec<Vec<Primitive>> = sorted.iter().map(|e| vec![e.clone()]).collect();
    crate::query::store_sort_keys(keys, vec![true]);
    sorted
}

// thenBy (compound sort: sort by new key, tiebreak by previous keys)
#[yaql_raw_function("thenBy", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn then_by_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut new_keys: Vec<Primitive> = Vec::new();
    for e in arr {
        new_keys.push(eval_lambda(lambda, e.clone())?);
    }
    let (prev_keys, prev_desc) = load_sort_keys();
    let has_prev = prev_keys.len() == new_keys.len();
    let mut indices: Vec<usize> = (0..new_keys.len()).collect();
    if has_prev {
        indices.sort_by(|&a, &b| {
            compare_key_vectors_with_desc(&prev_keys[a], &prev_keys[b], &prev_desc)
                .then_with(|| yaql_core::lang::compare(&new_keys[a], &new_keys[b]))
        });
    } else {
        indices.sort_by(|&a, &b| yaql_core::lang::compare(&new_keys[a], &new_keys[b]));
    }
    let sorted: Vec<Primitive> = indices.iter().map(|&i| arr[i].clone()).collect();
    let combined_keys: Vec<Vec<Primitive>> = indices.iter().map(|&i| {
        let mut all = prev_keys.get(i).cloned().unwrap_or_default();
        all.push(new_keys[i].clone());
        all
    }).collect();
    let mut combined_desc = prev_desc.clone();
    combined_desc.push(false);
    crate::query::store_sort_keys(combined_keys, combined_desc);
    Ok(Primitive::Array(sorted))
}
#[yaql_function("thenBy")]
fn then_by_identity(arr: Vec<Primitive>, _any: Any) -> Vec<Primitive> { arr }

#[yaql_raw_function("thenByDescending", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn then_by_desc_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut new_keys: Vec<Primitive> = Vec::new();
    for e in arr {
        new_keys.push(eval_lambda(lambda, e.clone())?);
    }
    let (prev_keys, prev_desc) = load_sort_keys();
    let has_prev = prev_keys.len() == new_keys.len();
    let mut indices: Vec<usize> = (0..new_keys.len()).collect();
    if has_prev {
        indices.sort_by(|&a, &b| {
            compare_key_vectors_with_desc(&prev_keys[a], &prev_keys[b], &prev_desc)
                .then_with(|| yaql_core::lang::compare(&new_keys[b], &new_keys[a]))
        });
    } else {
        indices.sort_by(|&a, &b| yaql_core::lang::compare(&new_keys[b], &new_keys[a]));
    }
    let sorted: Vec<Primitive> = indices.iter().map(|&i| arr[i].clone()).collect();
    let combined_keys: Vec<Vec<Primitive>> = indices.iter().map(|&i| {
        let mut all = prev_keys.get(i).cloned().unwrap_or_default();
        all.push(new_keys[i].clone());
        all
    }).collect();
    let mut combined_desc = prev_desc.clone();
    combined_desc.push(true);
    crate::query::store_sort_keys(combined_keys, combined_desc);
    Ok(Primitive::Array(sorted))
}
#[yaql_function("thenByDescending")]
fn then_by_desc_identity(arr: Vec<Primitive>, _any: Any) -> Vec<Primitive> { arr }

pub fn compare_key_vectors_with_desc(a: &[Primitive], b: &[Primitive], desc: &[bool]) -> std::cmp::Ordering {
    for (i, (ka, kb)) in a.iter().zip(b.iter()).enumerate() {
        let ord = if desc.get(i).copied().unwrap_or(false) {
            yaql_core::lang::compare(kb, ka)
        } else {
            yaql_core::lang::compare(ka, kb)
        };
        if ord != std::cmp::Ordering::Equal { return ord; }
    }
    a.len().cmp(&b.len())
}

// takeWhile
#[yaql_raw_function("takeWhile", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn take_while_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut result = Vec::new();
    for e in arr {
        if !yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?) { break; }
        result.push(e.clone());
    }
    Ok(Primitive::Array(result))
}
// skipWhile
#[yaql_raw_function("skipWhile", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn skip_while_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut result = Vec::new();
    let mut skipping = true;
    for e in arr {
        if skipping && yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?) { continue; }
        skipping = false;
        result.push(e.clone());
    }
    Ok(Primitive::Array(result))
}
// any with predicate
#[yaql_raw_function("any", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn any_pred_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    for e in arr {
        if yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?) {
            return Ok(Primitive::Boolean(true));
        }
    }
    Ok(Primitive::Boolean(false))
}
#[yaql_function("any")]
fn any_identity(arr: Vec<Primitive>, _any: Any) -> bool {
    !arr.is_empty()
}

// all with predicate
#[yaql_raw_function("all", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn all_pred_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    for e in arr {
        if !yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?) {
            return Ok(Primitive::Boolean(false));
        }
    }
    Ok(Primitive::Boolean(true))
}
#[yaql_function("all")]
fn all_identity(arr: Vec<Primitive>, _any: Any) -> bool {
    arr.iter().all(yaql_core::lang::truthy)
}

// distinct with selector
#[yaql_raw_function("distinct", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn distinct_sel_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut seen_keys = Vec::new();
    let mut result = Vec::new();
    for e in arr {
        let key = eval_lambda(lambda, e.clone())?;
        if !seen_keys.iter().any(|k| yaql_core::lang::primitive_eq(k, &key)) {
            seen_keys.push(key);
            result.push(e.clone());
        }
    }
    Ok(Primitive::Array(result))
}
#[yaql_function("distinct")]
fn distinct_identity(arr: Vec<Primitive>, _any: Any) -> Vec<Primitive> {
    let mut seen = Vec::new();
    for e in arr {
        yaql_core::lang::sets::set_push_unique(&mut seen, &e);
    }
    seen
}

// indexWhere
#[yaql_raw_function("indexWhere", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn index_where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    for (i, e) in arr.iter().enumerate() {
        if yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?) {
            return Ok(Primitive::Int(i as i64));
        }
    }
    Ok(Primitive::Int(-1))
}
#[yaql_function("indexWhere")]
fn index_where_identity(_arr: Vec<Primitive>, _any: Any) -> i64 { -1 }

// lastIndexWhere
#[yaql_raw_function("lastIndexWhere", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn last_index_where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    for (i, e) in arr.iter().enumerate().rev() {
        if yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?) {
            return Ok(Primitive::Int(i as i64));
        }
    }
    Ok(Primitive::Int(-1))
}
#[yaql_function("lastIndexWhere")]
fn last_index_where_identity(_arr: Vec<Primitive>, _any: Any) -> i64 { -1 }

// aggregate: array.aggregate(func) or array.aggregate(func, init)
#[yaql_raw_function("aggregate", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn aggregate_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    if arr.is_empty() { return Ok(Primitive::Null); }
    let mut acc = arr[0].clone();
    for e in &arr[1..] {
        acc = eval_lambda_2arg(lambda, acc, e.clone())?;
    }
    Ok(acc)
}
#[yaql_raw_function("aggregate", ArgSpec::Exact(3), [Type::Array, Type::Lambda, Type::Any], false)]
pub fn aggregate_init_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let init = args.get(2).cloned().unwrap_or(Primitive::Null);
    if arr.is_empty() { return Ok(init); }
    let mut acc = init;
    for e in arr {
        acc = eval_lambda_2arg(lambda, acc, e.clone())?;
    }
    Ok(acc)
}
// reduce (same as aggregate)
#[yaql_raw_function("reduce", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn reduce_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    if arr.is_empty() { return Ok(Primitive::Null); }
    let mut acc = arr[0].clone();
    for e in &arr[1..] {
        acc = eval_lambda_2arg(lambda, acc, e.clone())?;
    }
    Ok(acc)
}
#[yaql_raw_function("reduce", ArgSpec::Exact(3), [Type::Array, Type::Lambda, Type::Any], false)]
pub fn reduce_init_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let init = args.get(2).cloned().unwrap_or(Primitive::Null);
    if arr.is_empty() { return Ok(init); }
    let mut acc = init;
    for e in arr {
        acc = eval_lambda_2arg(lambda, acc, e.clone())?;
    }
    Ok(acc)
}
// accumulate
#[yaql_raw_function("accumulate", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn accumulate_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    if arr.is_empty() {
        Ok(Primitive::Array(Vec::new()))
    } else {
        let mut result = Vec::new();
        let mut acc = arr[0].clone();
        result.push(acc.clone());
        for e in &arr[1..] {
            acc = eval_lambda_2arg(lambda, acc, e.clone())?;
            result.push(acc.clone());
        }
        Ok(Primitive::Array(result))
    }
}
#[yaql_raw_function("accumulate", ArgSpec::Exact(3), [Type::Array, Type::Lambda, Type::Any], false)]
pub fn accumulate_init_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let init = args.get(2).cloned().unwrap_or(Primitive::Null);
    if arr.is_empty() {
        Ok(Primitive::Array(vec![init]))
    } else {
        let mut result = Vec::new();
        let mut acc = init;
        result.push(acc.clone());
        for e in arr {
            acc = eval_lambda_2arg(lambda, acc, e.clone())?;
            result.push(acc.clone());
        }
        Ok(Primitive::Array(result))
    }
}
// toDict: array.toDict(keyFunc, valueFunc) -> Map
#[yaql_raw_function("toDict", ArgSpec::Min(2), [Type::Array, Type::Any], false)]
pub fn to_dict_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let key_lambda = get_lambda(&args, 1);
    let val_lambda = get_lambda(&args, 2);
    let mut map = std::collections::HashMap::new();
    for e in arr {
        let key = if let Some(kl) = key_lambda { eval_lambda(kl, e.clone())? } else { e.clone() };
        let val = if let Some(vl) = val_lambda { eval_lambda(vl, e.clone())? } else { e.clone() };
        let key_str = match key {
            Primitive::String(s) => s,
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            _ => continue,
        };
        map.insert(key_str, val);
    }
    Ok(Primitive::Map(map))
}
// Evaluate a 2-arg lambda ($1 = acc, $2 = element, $ = element)
pub fn eval_lambda_2arg(lambda: &LambdaBody, acc: Primitive, element: Primitive) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let mut interp = yaql_core::interpreter::Interpreter { contexts: (*lambda.env).clone(), current_func: None };
    let elem_clone = element.clone();
    interp.push_context(element);
    interp.push_context(Primitive::Array(vec![acc, elem_clone]));
    yaql_core::interpreter::eval_body(&mut interp, &lambda.body)
}

// --- splitWhere: split at positions where predicate is true ---
#[yaql_raw_function("splitWhere", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn split_where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut result = Vec::new();
    let mut current = Vec::new();
    for e in arr {
        if yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?) {
            result.push(Primitive::Array(current));
            current = Vec::new();
        } else {
            current.push(e.clone());
        }
    }
    result.push(Primitive::Array(current));
    Ok(Primitive::Array(result))
}
// --- sliceWhere: group consecutive elements by predicate result ---
#[yaql_raw_function("sliceWhere", ArgSpec::Exact(2), [Type::Array, Type::Lambda], false)]
pub fn slice_where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(Primitive::Lambda(lambda)) = args.get(1) else { return Ok(Primitive::Null) };
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut last_pred: Option<bool> = None;
    for e in arr {
        let pred = yaql_core::lang::truthy(&eval_lambda(lambda, e.clone())?);
        if last_pred.is_some() && last_pred != Some(pred) {
            result.push(Primitive::Array(current));
            current = Vec::new();
        }
        current.push(e.clone());
        last_pred = Some(pred);
    }
    if !current.is_empty() { result.push(Primitive::Array(current)); }
    Ok(Primitive::Array(result))
}
// --- zip ---
#[yaql_raw_function("zip", ArgSpec::Min(2), [Type::Array], false)]
pub fn zip_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(first) = &args[0] else { return Ok(Primitive::Null) };
    let rest: Vec<&Vec<Primitive>> = args[1..].iter().filter_map(|a| if let Primitive::Array(a) = a { Some(a) } else { None }).collect();
    let min_len = first.len().min(rest.iter().map(|a| a.len()).min().unwrap_or(first.len()));
    let mut result = Vec::new();
    for i in 0..min_len {
        let mut tuple = vec![first[i].clone()];
        for r in &rest { tuple.push(r[i].clone()); }
        result.push(Primitive::Array(tuple));
    }
    Ok(Primitive::Array(result))
}
// --- zipLongest ---
#[yaql_raw_function("zipLongest", ArgSpec::Min(2), [Type::Array], true)]
pub fn zip_longest_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(first) = &args[0] else { return Ok(Primitive::Null) };
    let default = kwargs.iter().find_map(|(k, v)| {
        if let Primitive::String(key) = k { if key == "default" { return Some(v.clone()); } }
        None
    }).unwrap_or(Primitive::Null);
    let rest: Vec<&Vec<Primitive>> = args[1..].iter().filter_map(|a| if let Primitive::Array(a) = a { Some(a) } else { None }).collect();
    let max_len = first.len().max(rest.iter().map(|a| a.len()).max().unwrap_or(0));
    let mut result = Vec::new();
    for i in 0..max_len {
        let mut tuple = vec![first.get(i).cloned().unwrap_or(default.clone())];
        for r in &rest { tuple.push(r.get(i).cloned().unwrap_or(default.clone())); }
        result.push(Primitive::Array(tuple));
    }
    Ok(Primitive::Array(result))
}
// --- groupBy ---
fn group_by_impl(args: &[Primitive], kwargs: &[(Primitive, Primitive)]) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(arr) = &args[0] else { return Ok(Primitive::Null) };
    let Some(key_lambda) = get_lambda(args, 1) else { return Ok(Primitive::Null) };
    let sel_lambda = get_lambda(args, 2);
    let agg_lambda = if let Some(al) = get_lambda(args, 3) { Some(al) }
        else if let Some(Primitive::Lambda(al)) = kwargs.iter().find(|(k, _)| {
            if let Primitive::String(key) = k { key == "aggregator" } else { false }
        }).map(|(_, v)| v) { Some(al) } else { None };

    // Group by key
    let mut groups: Vec<(Primitive, Vec<Primitive>)> = Vec::new();
    for e in arr {
        let key = eval_lambda(key_lambda, e.clone())?;
        if let Some(pos) = groups.iter().position(|(k, _)| yaql_core::lang::primitive_eq(k, &key)) {
            groups[pos].1.push(e.clone());
        } else {
            groups.push((key, vec![e.clone()]));
        }
    }

    let mut result = Vec::new();
    for (key, group) in groups {
        let value = if let Some(al) = agg_lambda {
            let sel_group: Vec<Primitive> = if let Some(sl) = sel_lambda {
                group.iter().map(|e| eval_lambda(sl, e.clone())).collect::<Result<Vec<_>, _>>()?
            } else {
                group.clone()
            };
            eval_lambda(al, Primitive::Array(sel_group))?
        } else if let Some(sl) = sel_lambda {
            Primitive::Array(group.iter().map(|e| eval_lambda(sl, e.clone())).collect::<Result<Vec<_>, _>>()?)
        } else {
            Primitive::Array(group)
        };
        result.push(Primitive::Array(vec![key, value]));
    }
    Ok(Primitive::Array(result))
}

#[yaql_raw_function("groupBy", ArgSpec::Exact(2), [Type::Array, Type::Any], false)]
pub fn group_by_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    group_by_impl(&args, &[])
}
#[yaql_raw_function("groupBy", ArgSpec::Exact(3), [Type::Array, Type::Any, Type::Any], false)]
pub fn group_by_sel_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    group_by_impl(&args, &[])
}
#[yaql_raw_function("groupBy", ArgSpec::Exact(4), [Type::Array, Type::Any, Type::Any, Type::Any], false)]
pub fn group_by_agg_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    group_by_impl(&args, &[])
}
#[yaql_raw_function("groupBy", ArgSpec::Min(2), [Type::Array, Type::Any], true)]
pub fn group_by_kw_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    group_by_impl(&args, &kwargs)
}
// --- join ---
#[yaql_raw_function("join", ArgSpec::Min(3), [Type::Array, Type::Array, Type::Any], false)]
pub fn join_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Array(left) = &args[0] else { return Ok(Primitive::Null) };
    let Primitive::Array(right) = &args[1] else { return Ok(Primitive::Null) };
    let result_lambda = get_lambda(&args, 3);
    let mut result = Vec::new();
    for l in left {
        for r in right {
            let matches = if let Some(pred_lambda) = get_lambda(&args, 2) {
                yaql_core::lang::truthy(&eval_lambda_2arg(pred_lambda, l.clone(), r.clone())?)
            } else {
                yaql_core::lang::truthy(&args[2])
            };
            if matches {
                if let Some(rl) = result_lambda {
                    result.push(eval_lambda_2arg(rl, l.clone(), r.clone())?);
                } else {
                    result.push(Primitive::Array(vec![l.clone(), r.clone()]));
                }
            }
        }
    }
    Ok(Primitive::Array(result))
}
// --- mergeWith ---
#[yaql_raw_function("mergeWith", ArgSpec::Min(2), [Type::Map, Type::Map], true)]
pub fn merge_with_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let Primitive::Map(left) = &args[0] else { return Ok(Primitive::Null) };
    let Primitive::Map(right) = &args[1] else { return Ok(Primitive::Null) };
    let merge_lambda = get_lambda(&args, 2);
    let max_levels = kwargs.iter().find_map(|(k, v)| {
        if let Primitive::String(key) = k { if key == "maxLevels" {
            if let Primitive::Int(n) = v { return Some(*n as usize); }
        }}
        None
    }).unwrap_or(usize::MAX);

    fn merge_value(left: &Primitive, right: &Primitive, lambda: Option<&LambdaBody>, levels: usize) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
        if levels == 0 {
            return Ok(right.clone());
        }
        match (left, right) {
            (Primitive::Map(a), Primitive::Map(b)) => {
                let mut result = a.clone();
                for (k, v) in b {
                    if let Some(existing) = result.get(k) {
                        result.insert(k.clone(), merge_value(existing, v, lambda, levels - 1)?);
                    } else {
                        result.insert(k.clone(), v.clone());
                    }
                }
                Ok(Primitive::Map(result))
            }
            (Primitive::Array(a), Primitive::Array(b)) => {
                if let Some(ml) = lambda {
                    eval_lambda_2arg(ml, Primitive::Array(a.clone()), Primitive::Array(b.clone()))
                } else {
                    // Unique merge: append b elements not already in a
                    let mut combined = a.clone();
                    for e in b {
                        if !combined.iter().any(|x| yaql_core::lang::primitive_eq(x, e)) {
                            combined.push(e.clone());
                        }
                    }
                    Ok(Primitive::Array(combined))
                }
            }
            _ => Ok(right.clone()),
        }
    }

    let mut result = left.clone();
    for (k, v) in right {
        if let Some(existing) = result.get(k) {
            result.insert(k.clone(), merge_value(existing, v, merge_lambda, max_levels.saturating_sub(1))?);
        } else {
            result.insert(k.clone(), v.clone());
        }
    }
    Ok(Primitive::Map(result))
}
// --- generate ---
#[yaql_raw_function("generate", ArgSpec::Min(3), [Type::Any, Type::Any, Type::Any], false)]
pub fn generate_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let init = &args[0];
    let Some(cond_lambda) = get_lambda(&args, 1) else { return Ok(Primitive::Null) };
    let Some(next_lambda) = get_lambda(&args, 2) else { return Ok(Primitive::Null) };
    let proj_lambda = get_lambda(&args, 3);
    let mut result = Vec::new();
    let mut current = init.clone();
    let max_iters = 100_000;
    for _ in 0..max_iters {
        if !yaql_core::lang::truthy(&eval_lambda(cond_lambda, current.clone())?) { break; }
        if let Some(pl) = proj_lambda {
            result.push(eval_lambda(pl, current.clone())?);
        } else {
            result.push(current.clone());
        }
        current = eval_lambda(next_lambda, current)?;
    }
    Ok(Primitive::Array(result))
}
// --- repeat ---
// repeat(value, count) -> array of value repeated count times
// repeat(value) -> infinite (capped at 10000 for take/limit)
#[yaql_raw_function("repeat", ArgSpec::Min(1), [Type::Any], false)]
pub fn repeat_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    let value = args.get(0).cloned().unwrap_or(Primitive::Null);
    if args.len() > 1 {
        if let Primitive::Int(n) = &args[1] {
            return Ok(Primitive::Array((0..*n).map(|_| value.clone()).collect()));
        }
    }
    // Infinite repeat — cap at 10000
    Ok(Primitive::Array((0..10000).map(|_| value.clone()).collect()))
}
// --- cycle ---
// cycle(array) -> infinite cycling of array elements (capped at 10000)
#[yaql_function("cycle")]
fn cycle_fn(arr: Vec<Primitive>) -> Vec<Primitive> {
    if arr.is_empty() { Vec::new() }
    else { (0..10000).map(|i| arr[i % arr.len()].clone()).collect() }
}
