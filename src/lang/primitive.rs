use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct RegexWrapper(pub Arc<Regex>, pub bool);

impl RegexWrapper {
    pub fn new(re: Regex, ignore_case: bool) -> Self {
        RegexWrapper(Arc::new(re), ignore_case)
    }
}

impl PartialEq for RegexWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str() && self.1 == other.1
    }
}

#[derive(Clone, Debug)]
pub enum Primitive {
    String(String),
    Int(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Primitive>),
    Set(Vec<Primitive>),
    Map(HashMap<String, Primitive>),
    Regex(RegexWrapper),
    Null,
}

pub fn truthy(value: &Primitive) -> bool {
    match value {
        Primitive::String(s) => s.len() > 0,
        Primitive::Int(n) => *n != 0,
        Primitive::Float(n) => *n != 0.0,
        Primitive::Boolean(b) => *b,
        Primitive::Array(v) => v.len() > 0,
        Primitive::Set(v) => v.len() > 0,
        Primitive::Map(m) => m.len() > 0,
        Primitive::Regex(_) => true,
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
        (Primitive::Null, Primitive::Null) => Ordering::Equal,
        (Primitive::Null, _) => Ordering::Less,
        (_, Primitive::Null) => Ordering::Greater,
        (Primitive::Int(l), Primitive::Int(r)) => l.cmp(r),
        (Primitive::Float(l), Primitive::Float(r)) => l.partial_cmp(r).unwrap_or(Ordering::Equal),
        (Primitive::Int(l), Primitive::Float(r)) => (*l as f64).partial_cmp(r).unwrap_or(Ordering::Equal),
        (Primitive::Float(l), Primitive::Int(r)) => l.partial_cmp(&(*r as f64)).unwrap_or(Ordering::Equal),
        (Primitive::Boolean(l), Primitive::Boolean(r)) => l.cmp(r),
        (Primitive::String(l), Primitive::String(r)) => l.cmp(r),
        (Primitive::Array(l), Primitive::Array(r)) => {
            for (a, b) in l.iter().zip(r) {
                let ord = compare(a, b);
                if ord != Ordering::Equal { return ord; }
            }
            l.len().cmp(&r.len())
        }
        _ => Ordering::Equal,
    }
}

pub fn primitive_eq(a: &Primitive, b: &Primitive) -> bool {
    match (a, b) {
        (Primitive::String(l), Primitive::String(r)) => l == r,
        (Primitive::Int(l), Primitive::Int(r)) => l == r,
        (Primitive::Float(l), Primitive::Float(r)) => l == r,
        (Primitive::Int(l), Primitive::Float(r)) => *l as f64 == *r,
        (Primitive::Float(l), Primitive::Int(r)) => *l == *r as f64,
        (Primitive::Boolean(l), Primitive::Boolean(r)) => l == r,
        (Primitive::Int(l), Primitive::Boolean(r)) => (*l == 1) == *r,
        (Primitive::Boolean(l), Primitive::Int(r)) => *l == (*r == 1),
        (Primitive::Float(l), Primitive::Boolean(r)) => (*l == 1.0) == *r,
        (Primitive::Boolean(l), Primitive::Float(r)) => *l == (*r == 1.0),
        (Primitive::Null, Primitive::Null) => true,
        (Primitive::Array(l), Primitive::Array(r)) => l.len() == r.len() && l.iter().zip(r).all(|(a, b)| primitive_eq(a, b)),
        (Primitive::Set(l), Primitive::Set(r)) => crate::lang::sets::set_equal(l, r),
        (Primitive::Map(l), Primitive::Map(r)) => l.len() == r.len() && l.iter().all(|(k, v)| r.get(k).map_or(false, |rv| primitive_eq(v, rv))),
        _ => false,
    }
}

pub fn type_rank(p: &Primitive) -> u8 {
    match p {
        Primitive::Null => 0,
        Primitive::Array(_) => 1,
        Primitive::Int(_) | Primitive::Float(_) | Primitive::Boolean(_) => 2,
        Primitive::String(_) => 3,
        Primitive::Set(_) => 4,
        Primitive::Map(_) => 5,
        Primitive::Regex(_) => 6,
    }
}