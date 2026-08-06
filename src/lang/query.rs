use crate::lang::primitive::{Primitive, LambdaBody};
use crate::lang::functions::{FromPrimitive, IntoPrimitive, Any, Spec, SetVec};
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;
use crate::lang::functions::Type;
use crate::interpreter::eval_lambda;
use std::cell::RefCell;

// Thread-local storage for sort keys (for thenBy/thenByDescending)
// Each element's sort key is a Vec<Primitive> (multiple sort keys compound)
// SORT_DESC tracks whether each key position was sorted descending
thread_local! {
    static SORT_KEYS: RefCell<Vec<Vec<Primitive>>> = RefCell::new(Vec::new());
    static SORT_DESC: RefCell<Vec<bool>> = RefCell::new(Vec::new());
}

fn store_sort_keys(keys: Vec<Vec<Primitive>>, desc: Vec<bool>) {
    SORT_KEYS.with(|sk| { *sk.borrow_mut() = keys; });
    SORT_DESC.with(|sd| { *sd.borrow_mut() = desc; });
}

fn load_sort_keys() -> (Vec<Vec<Primitive>>, Vec<bool>) {
    let keys = SORT_KEYS.with(|sk| sk.borrow().clone());
    let desc = SORT_DESC.with(|sd| sd.borrow().clone());
    (keys, desc)
}

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
    let init = if args.len() > 1 {
        args[1].clone()
    } else if arr.first().map(|e| matches!(e, Primitive::String(_))).unwrap_or(false) {
        Primitive::String(String::new())
    } else if arr.first().map(|e| matches!(e, Primitive::Array(_))).unwrap_or(false) {
        Primitive::Array(Vec::new())
    } else {
        Primitive::Int(0)
    };
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

// --- Lambda-consuming functions ---

fn get_lambda(args: &[Primitive], idx: usize) -> Option<&LambdaBody> {
    if let Some(Primitive::Lambda(l)) = args.get(idx) { Some(l) } else { None }
}

// where: array.where(predicate) -> filtered array
pub fn where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let result: Vec<Primitive> = arr.iter()
        .filter_map(|e| {
            if crate::lang::truthy(&eval_lambda(lambda, e.clone())) {
                Some(e.clone())
            } else {
                None
            }
        })
        .collect();
    Primitive::Array(result)
}
yaql_raw_function!("where", where_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// select: array.select(selector) -> mapped array
pub fn select_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let result: Vec<Primitive> = arr.iter()
        .map(|e| eval_lambda(lambda, e.clone()))
        .collect();
    Primitive::Array(result)
}
yaql_raw_function!("select", select_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// selectMany: array.selectMany(selector) -> flatMap
pub fn select_many_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let lambda = get_lambda(&args, 1);
    let constant = args.get(1).cloned();
    let mut result = Vec::new();
    for e in arr {
        let val = if let Some(l) = lambda {
            eval_lambda(l, e.clone())
        } else if let Some(c) = &constant {
            c.clone()
        } else {
            continue;
        };
        match val {
            Primitive::Array(sub) => result.extend(sub),
            other => result.push(other),
        }
    }
    Primitive::Array(result)
}
yaql_raw_function!("selectMany", select_many_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// orderBy: array.orderBy(selector) -> sorted array (stores sort keys for thenBy)
pub fn order_by_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else {
        let mut sorted = arr.clone();
        sorted.sort_by(|a, b| crate::lang::compare(a, b));
        let keys: Vec<Vec<Primitive>> = sorted.iter().map(|e| vec![e.clone()]).collect();
        store_sort_keys(keys, vec![false]);
        return Primitive::Array(sorted);
    };
    let keyed: Vec<(Primitive, Primitive)> = arr.iter()
        .map(|e| (eval_lambda(lambda, e.clone()), e.clone()))
        .collect();
    let mut indices: Vec<usize> = (0..keyed.len()).collect();
    indices.sort_by(|&a, &b| crate::lang::compare(&keyed[a].0, &keyed[b].0));
    let sorted: Vec<Primitive> = indices.iter().map(|&i| keyed[i].1.clone()).collect();
    let sort_keys: Vec<Vec<Primitive>> = indices.iter().map(|&i| vec![keyed[i].0.clone()]).collect();
    store_sort_keys(sort_keys, vec![false]);
    Primitive::Array(sorted)
}
yaql_raw_function!("orderBy", order_by_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// orderByDescending
pub fn order_by_desc_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else {
        let mut sorted = arr.clone();
        sorted.sort_by(|a, b| crate::lang::compare(b, a));
        let keys: Vec<Vec<Primitive>> = sorted.iter().map(|e| vec![e.clone()]).collect();
        store_sort_keys(keys, vec![true]);
        return Primitive::Array(sorted);
    };
    let keyed: Vec<(Primitive, Primitive)> = arr.iter()
        .map(|e| (eval_lambda(lambda, e.clone()), e.clone()))
        .collect();
    let mut indices: Vec<usize> = (0..keyed.len()).collect();
    indices.sort_by(|&a, &b| crate::lang::compare(&keyed[b].0, &keyed[a].0));
    let sorted: Vec<Primitive> = indices.iter().map(|&i| keyed[i].1.clone()).collect();
    let sort_keys: Vec<Vec<Primitive>> = indices.iter().map(|&i| vec![keyed[i].0.clone()]).collect();
    store_sort_keys(sort_keys, vec![true]);
    Primitive::Array(sorted)
}
yaql_raw_function!("orderByDescending", order_by_desc_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// thenBy (compound sort: sort by new key, tiebreak by previous keys)
pub fn then_by_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Array(arr.clone()); };
    let new_keys: Vec<Primitive> = arr.iter().map(|e| eval_lambda(lambda, e.clone())).collect();
    let (prev_keys, prev_desc) = load_sort_keys();
    let has_prev = prev_keys.len() == arr.len();
    let mut indices: Vec<usize> = (0..arr.len()).collect();
    if has_prev {
        indices.sort_by(|&a, &b| {
            compare_key_vectors_with_desc(&prev_keys[a], &prev_keys[b], &prev_desc)
                .then_with(|| crate::lang::compare(&new_keys[a], &new_keys[b]))
        });
    } else {
        indices.sort_by(|&a, &b| crate::lang::compare(&new_keys[a], &new_keys[b]));
    }
    let sorted: Vec<Primitive> = indices.iter().map(|&i| arr[i].clone()).collect();
    let combined_keys: Vec<Vec<Primitive>> = indices.iter().map(|&i| {
        let mut all = prev_keys.get(i).cloned().unwrap_or_default();
        all.push(new_keys[i].clone());
        all
    }).collect();
    let mut combined_desc = prev_desc.clone();
    combined_desc.push(false);
    store_sort_keys(combined_keys, combined_desc);
    Primitive::Array(sorted)
}
yaql_raw_function!("thenBy", then_by_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

pub fn then_by_desc_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Array(arr.clone()); };
    let new_keys: Vec<Primitive> = arr.iter().map(|e| eval_lambda(lambda, e.clone())).collect();
    let (prev_keys, prev_desc) = load_sort_keys();
    let has_prev = prev_keys.len() == arr.len();
    let mut indices: Vec<usize> = (0..arr.len()).collect();
    if has_prev {
        indices.sort_by(|&a, &b| {
            compare_key_vectors_with_desc(&prev_keys[a], &prev_keys[b], &prev_desc)
                .then_with(|| crate::lang::compare(&new_keys[b], &new_keys[a]))
        });
    } else {
        indices.sort_by(|&a, &b| crate::lang::compare(&new_keys[b], &new_keys[a]));
    }
    let sorted: Vec<Primitive> = indices.iter().map(|&i| arr[i].clone()).collect();
    let combined_keys: Vec<Vec<Primitive>> = indices.iter().map(|&i| {
        let mut all = prev_keys.get(i).cloned().unwrap_or_default();
        all.push(new_keys[i].clone());
        all
    }).collect();
    let mut combined_desc = prev_desc.clone();
    combined_desc.push(true);
    store_sort_keys(combined_keys, combined_desc);
    Primitive::Array(sorted)
}
yaql_raw_function!("thenByDescending", then_by_desc_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

fn compare_key_vectors_with_desc(a: &[Primitive], b: &[Primitive], desc: &[bool]) -> std::cmp::Ordering {
    for (i, (ka, kb)) in a.iter().zip(b.iter()).enumerate() {
        let ord = if desc.get(i).copied().unwrap_or(false) {
            crate::lang::compare(kb, ka)
        } else {
            crate::lang::compare(ka, kb)
        };
        if ord != std::cmp::Ordering::Equal { return ord; }
    }
    a.len().cmp(&b.len())
}

// takeWhile
pub fn take_while_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let mut result = Vec::new();
    for e in arr {
        if !crate::lang::truthy(&eval_lambda(lambda, e.clone())) { break; }
        result.push(e.clone());
    }
    Primitive::Array(result)
}
yaql_raw_function!("takeWhile", take_while_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// skipWhile
pub fn skip_while_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let mut result = Vec::new();
    let mut skipping = true;
    for e in arr {
        if skipping && crate::lang::truthy(&eval_lambda(lambda, e.clone())) { continue; }
        skipping = false;
        result.push(e.clone());
    }
    Primitive::Array(result)
}
yaql_raw_function!("skipWhile", skip_while_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// any with predicate
pub fn any_pred_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Boolean(!arr.is_empty()); };
    Primitive::Boolean(arr.iter().any(|e| crate::lang::truthy(&eval_lambda(lambda, e.clone()))))
}
yaql_raw_function!("any", any_pred_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// all with predicate
pub fn all_pred_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Boolean(arr.iter().all(crate::lang::truthy)); };
    Primitive::Boolean(arr.iter().all(|e| crate::lang::truthy(&eval_lambda(lambda, e.clone()))))
}
yaql_raw_function!("all", all_pred_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// distinct with selector
pub fn distinct_sel_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else {
        let mut seen = Vec::new();
        for e in arr { crate::lang::sets::set_push_unique(&mut seen, e); }
        return Primitive::Array(seen);
    };
    let mut seen_keys = Vec::new();
    let mut result = Vec::new();
    for e in arr {
        let key = eval_lambda(lambda, e.clone());
        if !seen_keys.iter().any(|k| crate::lang::primitive_eq(k, &key)) {
            seen_keys.push(key);
            result.push(e.clone());
        }
    }
    Primitive::Array(result)
}
yaql_raw_function!("distinct", distinct_sel_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// indexWhere
pub fn index_where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Int(-1); };
    for (i, e) in arr.iter().enumerate() {
        if crate::lang::truthy(&eval_lambda(lambda, e.clone())) {
            return Primitive::Int(i as i64);
        }
    }
    Primitive::Int(-1)
}
yaql_raw_function!("indexWhere", index_where_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// lastIndexWhere
pub fn last_index_where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Int(-1); };
    for (i, e) in arr.iter().enumerate().rev() {
        if crate::lang::truthy(&eval_lambda(lambda, e.clone())) {
            return Primitive::Int(i as i64);
        }
    }
    Primitive::Int(-1)
}
yaql_raw_function!("lastIndexWhere", last_index_where_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// aggregate: array.aggregate(func) or array.aggregate(func, init)
pub fn aggregate_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    if arr.is_empty() { return Primitive::Null; }
    let mut acc = arr[0].clone();
    for e in &arr[1..] {
        acc = eval_lambda_2arg(lambda, acc, e.clone());
    }
    acc
}
yaql_raw_function!("aggregate", aggregate_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

pub fn aggregate_init_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let init = args.get(2).cloned().unwrap_or(Primitive::Null);
    if arr.is_empty() { return init; }
    let mut acc = init;
    for e in arr {
        acc = eval_lambda_2arg(lambda, acc, e.clone());
    }
    acc
}
yaql_raw_function!("aggregate", aggregate_init_fn, ArgSpec::Exact(3), [Type::Array, Type::Any, Type::Any], false);

// reduce (same as aggregate)
yaql_raw_function!("reduce", aggregate_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);
yaql_raw_function!("reduce", aggregate_init_fn, ArgSpec::Exact(3), [Type::Array, Type::Any, Type::Any], false);

// accumulate
pub fn accumulate_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    if arr.is_empty() { return Primitive::Array(vec![]); }
    let mut result = Vec::new();
    let mut acc = arr[0].clone();
    result.push(acc.clone());
    for e in &arr[1..] {
        acc = eval_lambda_2arg(lambda, acc, e.clone());
        result.push(acc.clone());
    }
    Primitive::Array(result)
}
yaql_raw_function!("accumulate", accumulate_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

pub fn accumulate_init_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let init = args.get(2).cloned().unwrap_or(Primitive::Null);
    let mut result = vec![init.clone()];
    let mut acc = init;
    for e in arr {
        acc = eval_lambda_2arg(lambda, acc, e.clone());
        result.push(acc.clone());
    }
    Primitive::Array(result)
}
yaql_raw_function!("accumulate", accumulate_init_fn, ArgSpec::Exact(3), [Type::Array, Type::Any, Type::Any], false);

// toDict: array.toDict(keyFunc, valueFunc) -> Map
pub fn to_dict_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let key_lambda = get_lambda(&args, 1);
    let val_lambda = get_lambda(&args, 2);
    let mut map = std::collections::HashMap::new();
    for e in arr {
        let key = if let Some(kl) = key_lambda { eval_lambda(kl, e.clone()) } else { e.clone() };
        let val = if let Some(vl) = val_lambda { eval_lambda(vl, e.clone()) } else { e.clone() };
        let key_str = match key {
            Primitive::String(s) => s,
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            _ => continue,
        };
        map.insert(key_str, val);
    }
    Primitive::Map(map)
}
yaql_raw_function!("toDict", to_dict_fn, ArgSpec::Min(2), [Type::Array, Type::Any], false);

// Evaluate a 2-arg lambda ($1 = acc, $2 = element, $ = element)
fn eval_lambda_2arg(lambda: &LambdaBody, acc: Primitive, element: Primitive) -> Primitive {
    let mut interp = crate::interpreter::Interpreter { contexts: lambda.env.clone(), current_func: None };
    let elem_clone = element.clone();
    interp.push_context(element);
    interp.push_context(Primitive::Array(vec![acc, elem_clone]));
    crate::interpreter::eval_body(&mut interp, &lambda.body)
}

// --- splitWhere: split at positions where predicate is true ---
pub fn split_where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let mut result = Vec::new();
    let mut current = Vec::new();
    for e in arr {
        if crate::lang::truthy(&eval_lambda(lambda, e.clone())) {
            result.push(Primitive::Array(current));
            current = Vec::new();
        } else {
            current.push(e.clone());
        }
    }
    result.push(Primitive::Array(current));
    Primitive::Array(result)
}
yaql_raw_function!("splitWhere", split_where_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// --- sliceWhere: group consecutive elements by predicate result ---
pub fn slice_where_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut last_pred: Option<bool> = None;
    for e in arr {
        let pred = crate::lang::truthy(&eval_lambda(lambda, e.clone()));
        if last_pred.is_some() && last_pred != Some(pred) {
            result.push(Primitive::Array(current));
            current = Vec::new();
        }
        current.push(e.clone());
        last_pred = Some(pred);
    }
    if !current.is_empty() { result.push(Primitive::Array(current)); }
    Primitive::Array(result)
}
yaql_raw_function!("sliceWhere", slice_where_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

// --- zip ---
pub fn zip_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(first) = &args[0] else { return Primitive::Null };
    let rest: Vec<&Vec<Primitive>> = args[1..].iter().filter_map(|a| if let Primitive::Array(a) = a { Some(a) } else { None }).collect();
    let min_len = first.len().min(rest.iter().map(|a| a.len()).min().unwrap_or(first.len()));
    let mut result = Vec::new();
    for i in 0..min_len {
        let mut tuple = vec![first[i].clone()];
        for r in &rest { tuple.push(r[i].clone()); }
        result.push(Primitive::Array(tuple));
    }
    Primitive::Array(result)
}
yaql_raw_function!("zip", zip_fn, ArgSpec::Min(2), [Type::Array], false);

// --- zipLongest ---
pub fn zip_longest_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(first) = &args[0] else { return Primitive::Null };
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
    Primitive::Array(result)
}
yaql_raw_function!("zipLongest", zip_longest_fn, ArgSpec::Min(2), [Type::Array], true);

// --- groupBy ---
fn group_by_impl(args: &[Primitive], kwargs: &[(Primitive, Primitive)]) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Some(key_lambda) = get_lambda(args, 1) else { return Primitive::Null };
    let sel_lambda = get_lambda(args, 2);
    let agg_lambda = if let Some(al) = get_lambda(args, 3) { Some(al) }
        else if let Some(Primitive::Lambda(al)) = kwargs.iter().find(|(k, _)| {
            if let Primitive::String(key) = k { key == "aggregator" } else { false }
        }).map(|(_, v)| v) { Some(al) } else { None };

    // Group by key
    let mut groups: Vec<(Primitive, Vec<Primitive>)> = Vec::new();
    for e in arr {
        let key = eval_lambda(key_lambda, e.clone());
        if let Some(pos) = groups.iter().position(|(k, _)| crate::lang::primitive_eq(k, &key)) {
            groups[pos].1.push(e.clone());
        } else {
            groups.push((key, vec![e.clone()]));
        }
    }

    let mut result = Vec::new();
    for (key, group) in groups {
        let value = if let Some(al) = agg_lambda {
            let sel_group: Vec<Primitive> = if let Some(sl) = sel_lambda {
                group.iter().map(|e| eval_lambda(sl, e.clone())).collect()
            } else {
                group.clone()
            };
            eval_lambda(al, Primitive::Array(sel_group))
        } else if let Some(sl) = sel_lambda {
            Primitive::Array(group.iter().map(|e| eval_lambda(sl, e.clone())).collect())
        } else {
            Primitive::Array(group)
        };
        result.push(Primitive::Array(vec![key, value]));
    }
    Primitive::Array(result)
}

pub fn group_by_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    group_by_impl(&args, &[])
}
yaql_raw_function!("groupBy", group_by_fn, ArgSpec::Exact(2), [Type::Array, Type::Any], false);

pub fn group_by_sel_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    group_by_impl(&args, &[])
}
yaql_raw_function!("groupBy", group_by_sel_fn, ArgSpec::Exact(3), [Type::Array, Type::Any, Type::Any], false);

pub fn group_by_agg_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    group_by_impl(&args, &[])
}
yaql_raw_function!("groupBy", group_by_agg_fn, ArgSpec::Exact(4), [Type::Array, Type::Any, Type::Any, Type::Any], false);

pub fn group_by_kw_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    group_by_impl(&args, &kwargs)
}
yaql_raw_function!("groupBy", group_by_kw_fn, ArgSpec::Min(2), [Type::Array, Type::Any], true);

// --- join ---
pub fn join_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(left) = &args[0] else { return Primitive::Null };
    let Primitive::Array(right) = &args[1] else { return Primitive::Null };
    let result_lambda = get_lambda(&args, 3);
    let mut result = Vec::new();
    for l in left {
        for r in right {
            let matches = if let Some(pred_lambda) = get_lambda(&args, 2) {
                crate::lang::truthy(&eval_lambda_2arg(pred_lambda, l.clone(), r.clone()))
            } else {
                crate::lang::truthy(&args[2])
            };
            if matches {
                if let Some(rl) = result_lambda {
                    result.push(eval_lambda_2arg(rl, l.clone(), r.clone()));
                } else {
                    result.push(Primitive::Array(vec![l.clone(), r.clone()]));
                }
            }
        }
    }
    Primitive::Array(result)
}
yaql_raw_function!("join", join_fn, ArgSpec::Min(3), [Type::Array, Type::Array, Type::Any], false);

// --- mergeWith ---
pub fn merge_with_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Map(left) = &args[0] else { return Primitive::Null };
    let Primitive::Map(right) = &args[1] else { return Primitive::Null };
    let merge_lambda = get_lambda(&args, 2);
    let max_levels = kwargs.iter().find_map(|(k, v)| {
        if let Primitive::String(key) = k { if key == "maxLevels" {
            if let Primitive::Int(n) = v { return Some(*n as usize); }
        }}
        None
    }).unwrap_or(usize::MAX);

    fn merge_value(left: &Primitive, right: &Primitive, lambda: Option<&LambdaBody>, levels: usize) -> Primitive {
        if levels == 0 {
            return right.clone();
        }
        match (left, right) {
            (Primitive::Map(a), Primitive::Map(b)) => {
                let mut result = a.clone();
                for (k, v) in b {
                    if let Some(existing) = result.get(k) {
                        result.insert(k.clone(), merge_value(existing, v, lambda, levels - 1));
                    } else {
                        result.insert(k.clone(), v.clone());
                    }
                }
                Primitive::Map(result)
            }
            (Primitive::Array(a), Primitive::Array(b)) => {
                if let Some(ml) = lambda {
                    eval_lambda_2arg(ml, Primitive::Array(a.clone()), Primitive::Array(b.clone()))
                } else {
                    // Unique merge: append b elements not already in a
                    let mut combined = a.clone();
                    for e in b {
                        if !combined.iter().any(|x| crate::lang::primitive_eq(x, e)) {
                            combined.push(e.clone());
                        }
                    }
                    Primitive::Array(combined)
                }
            }
            _ => right.clone(),
        }
    }

    let mut result = left.clone();
    for (k, v) in right {
        if let Some(existing) = result.get(k) {
            result.insert(k.clone(), merge_value(existing, v, merge_lambda, max_levels.saturating_sub(1)));
        } else {
            result.insert(k.clone(), v.clone());
        }
    }
    Primitive::Map(result)
}
yaql_raw_function!("mergeWith", merge_with_fn, ArgSpec::Min(2), [Type::Map, Type::Map], true);

// --- generate ---
pub fn generate_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let init = &args[0];
    let Some(cond_lambda) = get_lambda(&args, 1) else { return Primitive::Null };
    let Some(next_lambda) = get_lambda(&args, 2) else { return Primitive::Null };
    let proj_lambda = get_lambda(&args, 3);
    let mut result = Vec::new();
    let mut current = init.clone();
    let max_iters = 100_000;
    for _ in 0..max_iters {
        if !crate::lang::truthy(&eval_lambda(cond_lambda, current.clone())) { break; }
        if let Some(pl) = proj_lambda {
            result.push(eval_lambda(pl, current.clone()));
        } else {
            result.push(current.clone());
        }
        current = eval_lambda(next_lambda, current);
    }
    Primitive::Array(result)
}
yaql_raw_function!("generate", generate_fn, ArgSpec::Min(3), [Type::Any, Type::Any, Type::Any], false);

// --- repeat ---
// repeat(value, count) -> array of value repeated count times
// repeat(value) -> infinite (capped at 10000 for take/limit)
pub fn repeat_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let value = args.get(0).cloned().unwrap_or(Primitive::Null);
    if args.len() > 1 {
        if let Primitive::Int(n) = &args[1] {
            return Primitive::Array((0..*n).map(|_| value.clone()).collect());
        }
    }
    // Infinite repeat — cap at 10000
    Primitive::Array((0..10000).map(|_| value.clone()).collect())
}
yaql_raw_function!("repeat", repeat_fn, ArgSpec::Min(1), [Type::Any], false);

// --- cycle ---
// cycle(array) -> infinite cycling of array elements (capped at 10000)
pub fn cycle_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    if arr.is_empty() { return Primitive::Array(Vec::new()); }
    let result: Vec<Primitive> = (0..10000).map(|i| arr[i % arr.len()].clone()).collect();
    Primitive::Array(result)
}
yaql_raw_function!("cycle", cycle_fn, ArgSpec::Exact(1), [Type::Array], false);