use crate::lang::primitive::{Primitive, primitive_eq};
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;
use crate::lang::functions::Type;

pub fn set_push_unique(vec: &mut Vec<Primitive>, item: &Primitive) {
    if !vec.iter().any(|e| primitive_eq(e, item)) {
        vec.push(item.clone());
    }
}

pub fn is_subset(subset: &[Primitive], superset: &[Primitive]) -> bool {
    subset.iter().all(|e| superset.iter().any(|s| primitive_eq(e, s)))
}

pub fn set_equal(a: &[Primitive], b: &[Primitive]) -> bool {
    a.len() == b.len() && is_subset(a, b)
}

pub fn set_difference(a: &[Primitive], b: &[Primitive]) -> Vec<Primitive> {
    a.iter().filter(|e| !b.iter().any(|s| primitive_eq(e, s))).cloned().collect()
}

pub fn set_symmetric_difference(a: &[Primitive], b: &[Primitive]) -> Vec<Primitive> {
    let mut result = set_difference(a, b);
    for e in b {
        if !a.iter().any(|s| primitive_eq(s, e)) {
            set_push_unique(&mut result, e);
        }
    }
    result
}

pub fn set_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut seen = Vec::new();
    for arg in args {
        set_push_unique(&mut seen, &arg);
    }
    Primitive::Set(seen)
}
yaql_raw_function!("set", set_fn, ArgSpec::Varargs, [], false);

yaql_function!("union", union_ss(l: SetVec, r: SetVec) -> SetVec {
    let mut c = l.0;
    for e in &r.0 { crate::lang::sets::set_push_unique(&mut c, e); }
    SetVec(c)
});
yaql_function!("union", union_sa(l: SetVec, r: Vec<Primitive>) -> Vec<Primitive> {
    let mut c = l.0; c.extend(r); c
});
yaql_function!("union", union_as(l: Vec<Primitive>, r: SetVec) -> Vec<Primitive> {
    let mut c = l; c.extend(r.0); c
});

yaql_function!("difference", diff_ss(l: SetVec, r: SetVec) -> SetVec {
    SetVec(crate::lang::sets::set_difference(&l.0, &r.0))
});
yaql_function!("difference", diff_sa(l: SetVec, r: Vec<Primitive>) -> SetVec {
    SetVec(crate::lang::sets::set_difference(&l.0, &r))
});

yaql_function!("symmetricDifference", symdiff_ss(l: SetVec, r: SetVec) -> SetVec {
    SetVec(crate::lang::sets::set_symmetric_difference(&l.0, &r.0))
});
yaql_function!("symmetricDifference", symdiff_sa(l: SetVec, r: Vec<Primitive>) -> SetVec {
    SetVec(crate::lang::sets::set_symmetric_difference(&l.0, &r))
});

pub fn set_add(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Set(mut elems) = args[0].clone() else { return Primitive::Null };
    for arg in &args[1..] {
        set_push_unique(&mut elems, arg);
    }
    Primitive::Set(elems)
}
yaql_raw_function!("add", set_add, ArgSpec::Min(2), [Type::Set], false);

pub fn set_remove(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Set(elems) = &args[0] else { return Primitive::Null };
    let to_remove = &args[1..];
    let result: Vec<Primitive> = elems.iter()
        .filter(|e| !to_remove.iter().any(|r| primitive_eq(e, r)))
        .cloned()
        .collect();
    Primitive::Set(result)
}
yaql_raw_function!("remove", set_remove, ArgSpec::Min(2), [Type::Set], false);

yaql_function!("contains", set_contains(elems: SetVec, item: Any) -> bool {
    elems.0.iter().any(|e| crate::lang::primitive::primitive_eq(e, &item.0))
});

yaql_function!("toSet", to_set_fn(arr: Vec<Primitive>) -> SetVec {
    let mut seen = Vec::new();
    for e in &arr {
        crate::lang::sets::set_push_unique(&mut seen, e);
    }
    SetVec(seen)
});

yaql_function!("toSet", to_set_from_set(arr: SetVec) -> SetVec {
    arr
});

yaql_function!("isSet", is_set_fn(v: Any) -> bool {
    matches!(v.0, Primitive::Set(_))
});