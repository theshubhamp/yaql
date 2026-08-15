use yaql_core::lang::Primitive;
use yaql_core::lang::functions::EvalError;
use yaql_core::lang::functions::ArgSpec;
use yaql_core::lang::functions::Type;
use yaql_macros::yaql_function;

#[yaql_function("set", ArgSpec::Varargs, [], false)]
pub fn set_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let mut seen = Vec::new();
    for arg in args {
        yaql_core::lang::set_push_unique(&mut seen, &arg);
    }
    Ok(Primitive::Set(seen))
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

#[yaql_function("add", ArgSpec::Min(2), [Type::Set], false)]
pub fn set_add(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let Primitive::Set(mut elems) = args[0].clone() else { return Ok(Primitive::Null) };
    for arg in &args[1..] {
        yaql_core::lang::set_push_unique(&mut elems, arg);
    }
    Ok(Primitive::Set(elems))
}
#[yaql_function("remove", ArgSpec::Min(2), [Type::Set], false)]
pub fn set_remove(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let Primitive::Set(elems) = &args[0] else { return Ok(Primitive::Null) };
    let to_remove = &args[1..];
    let result: Vec<Primitive> = elems.iter()
        .filter(|e| !to_remove.iter().any(|r| yaql_core::lang::primitive_eq(e, r)))
        .cloned()
        .collect();
    Ok(Primitive::Set(result))
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
