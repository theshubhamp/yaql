use yaql_core::lang::Primitive;
use yaql_core::lang::functions::{SetVec, Varargs, Any};
use yaql_macros::yaql_function;

#[yaql_function("set")]
pub fn set_fn(args: Varargs<0>) -> SetVec {
    let mut seen = Vec::new();
    for arg in args.0 {
        yaql_core::lang::set_push_unique(&mut seen, &arg);
    }
    SetVec(seen)
}
#[yaql_function("union")]
fn union_ss(l: SetVec, r: SetVec) -> SetVec {
    let mut c = l.0;
    for e in &r.0 { yaql_core::lang::set_push_unique(&mut c, e); }
    SetVec(c)
}

#[yaql_function("union")]
fn union_sa(l: SetVec, r: Vec<Primitive>) -> Vec<Primitive> {
    let mut c = l.0; c.extend(r); c
}

#[yaql_function("union")]
fn union_as(l: Vec<Primitive>, r: SetVec) -> Vec<Primitive> {
    let mut c = l; c.extend(r.0); c
}

#[yaql_function("difference")]
fn diff_ss(l: SetVec, r: SetVec) -> SetVec {
    SetVec(yaql_core::lang::set_difference(&l.0, &r.0))
}

#[yaql_function("difference")]
fn diff_sa(l: SetVec, r: Vec<Primitive>) -> SetVec {
    SetVec(yaql_core::lang::set_difference(&l.0, &r))
}

#[yaql_function("symmetricDifference")]
fn symdiff_ss(l: SetVec, r: SetVec) -> SetVec {
    SetVec(yaql_core::lang::set_symmetric_difference(&l.0, &r.0))
}

#[yaql_function("symmetricDifference")]
fn symdiff_sa(l: SetVec, r: Vec<Primitive>) -> SetVec {
    SetVec(yaql_core::lang::set_symmetric_difference(&l.0, &r))
}

#[yaql_function("add")]
pub fn set_add(elems: SetVec, rest: Varargs<1>) -> SetVec {
    let mut elems = elems.0;
    for arg in &rest.0 {
        yaql_core::lang::set_push_unique(&mut elems, arg);
    }
    SetVec(elems)
}
#[yaql_function("remove")]
pub fn set_remove(elems: SetVec, rest: Varargs<1>) -> SetVec {
    let to_remove = &rest.0;
    let result: Vec<Primitive> = elems.0.iter()
        .filter(|e| !to_remove.iter().any(|r| yaql_core::lang::primitive_eq(e, r)))
        .cloned()
        .collect();
    SetVec(result)
}
#[yaql_function("contains")]
fn set_contains(elems: SetVec, item: Any) -> bool {
    elems.0.iter().any(|e| yaql_core::lang::primitive_eq(e, &item.0))
}

#[yaql_function("toSet")]
fn to_set_fn(arr: Vec<Primitive>) -> SetVec {
    let mut seen = Vec::new();
    for e in &arr {
        yaql_core::lang::set_push_unique(&mut seen, e);
    }
    SetVec(seen)
}

#[yaql_function("toSet")]
fn to_set_from_set(arr: SetVec) -> SetVec {
    arr
}

#[yaql_function("isSet")]
fn is_set_fn(v: Any) -> bool {
    matches!(v.0, Primitive::Set(_))
}
