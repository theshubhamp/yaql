use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Primitive {
    String(String),
    Int(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Primitive>),
    Map(HashMap<String, Primitive>),
    Null,
}

pub fn truthy(value: &Primitive) -> bool {
    match value {
        Primitive::String(s) => s.len() > 0,
        Primitive::Int(n) => *n != 0,
        Primitive::Float(n) => *n != 0.0,
        Primitive::Boolean(b) => *b,
        Primitive::Array(v) => v.len() > 0,
        Primitive::Map(m) => m.len() > 0,
        Primitive::Null => false,
    }
}

pub fn as_f64(p: &Primitive) -> Option<f64> {
    match p {
        Primitive::Int(n) => Some(*n as f64),
        Primitive::Float(n) => Some(*n),
        _ => None,
    }
}

pub fn arith(left: &Primitive, right: &Primitive, f: fn(f64, f64) -> f64, i: fn(i64, i64) -> i64) -> Primitive {
    match (left, right) {
        (Primitive::Int(l), Primitive::Int(r)) => Primitive::Int(i(*l, *r)),
        _ => match (as_f64(left), as_f64(right)) {
            (Some(l), Some(r)) => Primitive::Float(f(l, r)),
            _ => Primitive::Null,
        },
    }
}

pub fn compare(a: &Primitive, b: &Primitive) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (Primitive::Int(l), Primitive::Int(r)) => l.cmp(r),
        (Primitive::Float(l), Primitive::Float(r)) => l.partial_cmp(r).unwrap_or(Ordering::Equal),
        (Primitive::Int(l), Primitive::Float(r)) => (*l as f64).partial_cmp(r).unwrap_or(Ordering::Equal),
        (Primitive::Float(l), Primitive::Int(r)) => l.partial_cmp(&(*r as f64)).unwrap_or(Ordering::Equal),
        (Primitive::String(l), Primitive::String(r)) => l.cmp(r),
        (Primitive::Boolean(l), Primitive::Boolean(r)) => l.cmp(r),
        _ => Ordering::Equal,
    }
}

pub fn primitive_eq(a: &Primitive, b: &Primitive) -> bool {
    matches!(super::operators::eq(a.clone(), b.clone()), Primitive::Boolean(true))
}