use crate::lang::{primitive_eq, Primitive};

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
