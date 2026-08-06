use crate::lang::primitive::{Primitive, as_f64, arith, truthy, primitive_eq};
use crate::lang::sets::{set_push_unique, is_subset, set_equal, set_difference};
use crate::lang::regex::{match_op, not_match_op};

pub fn add(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::String(l), Primitive::String(r)) => Primitive::String(format!("{}{}", l, r)),
        (Primitive::Array(l), Primitive::Array(r)) => {
            let mut combined = l.clone();
            combined.extend(r.iter().cloned());
            Primitive::Array(combined)
        }
        (Primitive::Set(l), Primitive::Set(r)) => {
            let mut combined = l.clone();
            for e in r {
                set_push_unique(&mut combined, e);
            }
            Primitive::Set(combined)
        }
        (Primitive::Set(l), Primitive::Array(r)) => {
            let mut combined = l.clone();
            combined.extend(r.iter().cloned());
            Primitive::Array(combined)
        }
        (Primitive::Array(l), Primitive::Set(r)) => {
            let mut combined = l.clone();
            combined.extend(r.iter().cloned());
            Primitive::Array(combined)
        }
        (Primitive::Map(l), Primitive::Map(r)) => {
            let mut combined = l.clone();
            for (k, v) in r {
                combined.insert(k.clone(), v.clone());
            }
            Primitive::Map(combined)
        }
        _ => arith(&left, &right, |a, b| a + b, |a, b| a + b),
    }
}

pub fn sub(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::Set(l), Primitive::Set(r)) => Primitive::Set(set_difference(l, r)),
        _ => arith(&left, &right, |a, b| a - b, |a, b| a - b),
    }
}

pub fn mul(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::String(s), Primitive::Int(n)) => Primitive::String(s.repeat(*n as usize)),
        (Primitive::Int(n), Primitive::String(s)) => Primitive::String(s.repeat(*n as usize)),
        (Primitive::Array(a), Primitive::Int(n)) => {
            Primitive::Array((0..*n).flat_map(|_| a.iter().cloned()).collect())
        }
        (Primitive::Int(n), Primitive::Array(a)) => {
            Primitive::Array((0..*n).flat_map(|_| a.iter().cloned()).collect())
        }
        _ => arith(&left, &right, |a, b| a * b, |a, b| a * b),
    }
}

pub fn div(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::Int(_), Primitive::Int(0)) => panic!("division by zero"),
        (Primitive::Int(_), Primitive::Float(0.0)) => panic!("division by zero"),
        (Primitive::Float(_), Primitive::Int(0)) => panic!("division by zero"),
        (Primitive::Float(_), Primitive::Float(0.0)) => panic!("division by zero"),
        _ => arith(&left, &right, |a, b| a / b, |a, b| (a as f64 / b as f64).floor() as i64),
    }
}

pub fn modulo(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::Int(l), Primitive::Int(r)) => Primitive::Int(*l - r * ((*l as f64 / *r as f64).floor() as i64)),
        _ => match (as_f64(&left), as_f64(&right)) {
            (Some(l), Some(r)) => Primitive::Float(l - r * (l / r).floor()),
            _ => Primitive::Null,
        },
    }
}

pub fn and(left: Primitive, right: Primitive) -> Primitive {
    if !truthy(&left) { return left }
    right
}

pub fn or(left: Primitive, right: Primitive) -> Primitive {
    if truthy(&left) { return left }
    right
}

pub fn eq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::String(l), Primitive::String(r)) => l == r,
        (Primitive::Int(l), Primitive::Int(r)) => l == r,
        (Primitive::Float(l), Primitive::Float(r)) => l == r,
        (Primitive::Int(l), Primitive::Float(r)) => *l as f64 == *r,
        (Primitive::Float(l), Primitive::Int(r)) => *l == *r as f64,
        (Primitive::Int(l), Primitive::Boolean(r)) => (*l == 1) == *r,
        (Primitive::Boolean(l), Primitive::Int(r)) => *l == (*r == 1),
        (Primitive::Float(l), Primitive::Boolean(r)) => (*l == 1.0) == *r,
        (Primitive::Boolean(l), Primitive::Float(r)) => *l == (*r == 1.0),
        (Primitive::Boolean(l), Primitive::Boolean(r)) => l == r,
        (Primitive::Null, Primitive::Null) => true,
        (Primitive::Array(l), Primitive::Array(r)) => l.len() == r.len() && l.iter().zip(r).all(|(a, b)| primitive_eq(a, b)),
        (Primitive::Set(l), Primitive::Set(r)) => set_equal(l, r),
        (Primitive::Map(l), Primitive::Map(r)) => l.len() == r.len() && l.iter().all(|(k, v)| r.get(k).map_or(false, |rv| primitive_eq(v, rv))),
        _ => false
    };
    Primitive::Boolean(result)
}

pub fn neq(left: Primitive, right: Primitive) -> Primitive {
    let Primitive::Boolean(is_eq) = eq(left, right) else { return Primitive::Boolean(true) };
    Primitive::Boolean(!is_eq)
}

pub fn lt(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::Set(l), Primitive::Set(r)) => {
            l.len() < r.len() && is_subset(l, r)
        }
        (Primitive::Null, Primitive::Null) => false,
        (Primitive::Null, _) => true,
        (_, Primitive::Null) => false,
        _ => match (as_f64(&left), as_f64(&right)) {
            (Some(l), Some(r)) => l < r,
            _ => false,
        },
    };
    Primitive::Boolean(result)
}

pub fn lteq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::Set(l), Primitive::Set(r)) => is_subset(l, r),
        (Primitive::Null, Primitive::Null) => true,
        (Primitive::Null, _) => true,
        (_, Primitive::Null) => false,
        _ => match (as_f64(&left), as_f64(&right)) {
            (Some(l), Some(r)) => l <= r,
            _ => false,
        },
    };
    Primitive::Boolean(result)
}

pub fn gt(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::Set(l), Primitive::Set(r)) => {
            l.len() > r.len() && is_subset(r, l)
        }
        (Primitive::Null, Primitive::Null) => false,
        (Primitive::Null, _) => false,
        (_, Primitive::Null) => true,
        _ => match (as_f64(&left), as_f64(&right)) {
            (Some(l), Some(r)) => l > r,
            _ => false,
        },
    };
    Primitive::Boolean(result)
}

pub fn gteq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::Set(l), Primitive::Set(r)) => is_subset(r, l),
        (Primitive::Null, Primitive::Null) => true,
        (Primitive::Null, _) => false,
        (_, Primitive::Null) => true,
        _ => match (as_f64(&left), as_f64(&right)) {
            (Some(l), Some(r)) => l >= r,
            _ => false,
        },
    };
    Primitive::Boolean(result)
}

pub fn in_op(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::String(needle), Primitive::String(haystack)) => haystack.contains(needle.as_str()),
        (_, Primitive::Array(arr, ..)) => arr.iter().any(|e| primitive_eq(e, &left)),
        (_, Primitive::Set(arr)) => arr.iter().any(|e| primitive_eq(e, &left)),
        _ => false,
    };
    Primitive::Boolean(result)
}

pub fn dot_access(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::Map(map), Primitive::String(key)) => map.get(key).cloned().unwrap_or(Primitive::Null),
        _ => Primitive::Null,
    }
}

pub struct BinaryOperators;

impl BinaryOperators {
    pub fn lookup(&self, name: String) -> fn(Primitive, Primitive) -> Primitive {
        match name.as_str() {
            "and" => and,
            "or" => or,
            "+" => add,
            "-" => sub,
            "*" => mul,
            "/" => div,
            "mod" => modulo,
            "=" => eq,
            "!=" => neq,
            "<" => lt,
            "<=" => lteq,
            ">" => gt,
            ">=" => gteq,
            "in" => in_op,
            "." => dot_access,
            "?." => dot_access,
            "=~" => match_op,
            "!~" => not_match_op,
            "=>" => |_, _| Primitive::Null,
            _ => todo!()
        }
    }
}

pub static BINARY_OPERATORS: BinaryOperators = BinaryOperators {};