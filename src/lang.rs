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
    return match value {
        Primitive::String(string) => string.len() > 0,
        Primitive::Int(num) => *num != 0,
        Primitive::Float(num) => *num != 0.0,
        Primitive::Boolean(bool) => *bool,
        Primitive::Array(vec) => vec.len() > 0,
        Primitive::Map(map) => map.len() > 0,
        Primitive::Null => false,
    }
}

fn as_f64(p: &Primitive) -> Option<f64> {
    match p {
        Primitive::Int(n) => Some(*n as f64),
        Primitive::Float(n) => Some(*n),
        _ => None,
    }
}

/// Result is Float if either operand is Float, Integer if both are Integer.
fn arith(left: &Primitive, right: &Primitive, f: fn(f64, f64) -> f64, i: fn(i64, i64) -> i64) -> Primitive {
    match (left, right) {
        (Primitive::Int(l), Primitive::Int(r)) => Primitive::Int(i(*l, *r)),
        _ => match (as_f64(left), as_f64(right)) {
            (Some(l), Some(r)) => Primitive::Float(f(l, r)),
            _ => Primitive::Null,
        },
    }
}

pub fn add(left: Primitive, right: Primitive) -> Primitive {
    arith(&left, &right, |a, b| a + b, |a, b| a + b)
}

pub fn sub(left: Primitive, right: Primitive) -> Primitive {
    arith(&left, &right, |a, b| a - b, |a, b| a - b)
}

pub fn mul(left: Primitive, right: Primitive) -> Primitive {
    arith(&left, &right, |a, b| a * b, |a, b| a * b)
}

pub fn div(left: Primitive, right: Primitive) -> Primitive {
    arith(&left, &right, |a, b| a / b, |a, b| (a as f64 / b as f64).floor() as i64)
}

pub fn modulo(left: Primitive, right: Primitive) -> Primitive {
    arith(&left, &right, |a, b| a % b, |a, b| a - b * (a as f64 / b as f64).floor() as i64)
}

pub fn and(left: Primitive, right: Primitive) -> Primitive {
    if !truthy(&left) {
        return left
    }

    return right
}

pub fn or(left: Primitive, right: Primitive) -> Primitive {
    if truthy(&left) {
        return left
    }

    return right
}

pub fn eq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (left, right) {
        (Primitive::String(l), Primitive::String(r)) => l == r,
        (Primitive::Int(l), Primitive::Int(r)) => l == r,
        (Primitive::Float(l), Primitive::Float(r)) => l == r,
        (Primitive::Int(l), Primitive::Float(r)) => l as f64 == r,
        (Primitive::Float(l), Primitive::Int(r)) => l == r as f64,
        (Primitive::Boolean(l), Primitive::Boolean(r)) => l == r,
        (Primitive::Null, Primitive::Null) => true,
        _ => false
    };

    return Primitive::Boolean(result);
}

pub fn neq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (left, right) {
        (Primitive::String(l), Primitive::String(r)) => l != r,
        (Primitive::Int(l), Primitive::Int(r)) => l != r,
        (Primitive::Float(l), Primitive::Float(r)) => l != r,
        (Primitive::Int(l), Primitive::Float(r)) => l as f64 != r,
        (Primitive::Float(l), Primitive::Int(r)) => l != r as f64,
        (Primitive::Boolean(l), Primitive::Boolean(r)) => l != r,
        (Primitive::Null, Primitive::Null) => false,
        _ => true
    };

    return Primitive::Boolean(result);
}

pub fn lt(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::Null, Primitive::Null) => false,
        (Primitive::Null, _) => true,
        (_, Primitive::Null) => false,
        _ => match (as_f64(&left), as_f64(&right)) {
            (Some(l), Some(r)) => l < r,
            _ => false,
        },
    };

    return Primitive::Boolean(result);
}

pub fn lteq(left: Primitive, right: Primitive) -> Primitive {
    let Primitive::Boolean(lt) = lt(left.clone(), right.clone()) else {
        return Primitive::Boolean(false);
    };
    let Primitive::Boolean(eq) = eq(left, right) else {
        return Primitive::Boolean(false);
    };
    Primitive::Boolean(lt || eq)
}

pub fn gt(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::Null, Primitive::Null) => false,
        (Primitive::Null, _) => false,
        (_, Primitive::Null) => true,
        _ => match (as_f64(&left), as_f64(&right)) {
            (Some(l), Some(r)) => l > r,
            _ => false,
        },
    };

    return Primitive::Boolean(result);
}

pub fn gteq(left: Primitive, right: Primitive) -> Primitive {
    let Primitive::Boolean(gt) = gt(left.clone(), right.clone()) else {
        return Primitive::Boolean(false);
    };
    let Primitive::Boolean(eq) = eq(left, right) else {
        return Primitive::Boolean(false);
    };
    Primitive::Boolean(gt || eq)
}

pub struct BinaryOperators;

impl BinaryOperators {
    pub fn lookup(&self, name: String) -> fn(Primitive, Primitive) -> Primitive {
        return match name.as_str() {
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
            _ => todo!()
        }
    }
}

pub static BINARY_OPERATORS: BinaryOperators = BinaryOperators {};

pub fn switch(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(args.len(), 0);

    for (switch_case, switch_mapping) in kwargs {
        if truthy(&switch_case) {
            return switch_mapping;
        }
    }

    return Primitive::Null;
}

pub fn select_case(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);

    let mut index = 0;
    for predicate in args {
        if truthy(&predicate) {
            return Primitive::Int(index);
        }

        index += 1;
    }

    return Primitive::Int(index);
}

pub fn select_all_cases(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);

    let mut cases = Vec::new();
    for (index, predicate) in args.iter().enumerate() {
        if truthy(predicate) {
            cases.push(Primitive::Int(index as i64));
        }
    }

    return Primitive::Array(cases);
}

pub fn examine(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);

    let mut cases = Vec::new();
    for predicate in args {
        if truthy(&predicate) {
            cases.push(Primitive::Boolean(true))
        } else {
            cases.push(Primitive::Boolean(false))
        }
    }

    return Primitive::Array(cases);
}

pub fn max(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    let mut result: Option<Primitive> = None;
    for arg in args {
        result = Some(match (result, arg) {
            (None, a) | (Some(Primitive::Null), a) => a,
            (r, Primitive::Null) => r.unwrap(),
            (Some(r), a) => if as_f64(&r) >= as_f64(&a) { r } else { a },
        });
    }
    result.unwrap_or(Primitive::Null)
}

pub fn min(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    let mut result: Option<Primitive> = None;
    for arg in args {
        result = Some(match (result, arg) {
            (None, a) => a,
            (Some(Primitive::Null), _) => Primitive::Null,
            (r, Primitive::Null) => r.unwrap(),
            (Some(r), a) => if as_f64(&r) <= as_f64(&a) { r } else { a },
        });
    }
    result.unwrap_or(Primitive::Null)
}

pub fn is_boolean(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    Primitive::Boolean(matches!(args[0], Primitive::Boolean(_)))
}

pub fn coalesce(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    for arg in args {
        if !matches!(arg, Primitive::Null) {
            return arg;
        }
    }
    Primitive::Null
}

pub struct Functions;

impl Functions {
    pub fn lookup(&self, name: String) -> fn(Vec<Primitive>, Vec<(Primitive, Primitive)>) -> Primitive {
        return match name.as_str() {
            "switch" => switch,
            "selectCase" => select_case,
            "selectAllCases" => select_all_cases,
            "examine" => examine,
            "isBoolean" => is_boolean,
            "coalesce" => coalesce,
            "max" => max,
            "min" => min,
            _ => todo!()
        }
    }
}

pub static FUNCTIONS: Functions = Functions {};