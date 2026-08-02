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
    match (&left, &right) {
        (Primitive::String(l), Primitive::String(r)) => Primitive::String(format!("{}{}", l, r)),
        (Primitive::Array(l), Primitive::Array(r)) => {
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
    arith(&left, &right, |a, b| a - b, |a, b| a - b)
}

pub fn mul(left: Primitive, right: Primitive) -> Primitive {
    match (&left, &right) {
        (Primitive::String(s), Primitive::Int(n)) => Primitive::String(s.repeat(*n as usize)),
        (Primitive::Int(n), Primitive::String(s)) => Primitive::String(s.repeat(*n as usize)),
        _ => arith(&left, &right, |a, b| a * b, |a, b| a * b),
    }
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

fn primitive_eq(a: &Primitive, b: &Primitive) -> bool {
    matches!(eq(a.clone(), b.clone()), Primitive::Boolean(true))
}

pub fn eq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::String(l), Primitive::String(r)) => l == r,
        (Primitive::Int(l), Primitive::Int(r)) => l == r,
        (Primitive::Float(l), Primitive::Float(r)) => l == r,
        (Primitive::Int(l), Primitive::Float(r)) => *l as f64 == *r,
        (Primitive::Float(l), Primitive::Int(r)) => *l == *r as f64,
        (Primitive::Boolean(l), Primitive::Boolean(r)) => l == r,
        (Primitive::Null, Primitive::Null) => true,
        (Primitive::Array(l), Primitive::Array(r)) => l.len() == r.len() && l.iter().zip(r).all(|(a, b)| primitive_eq(a, b)),
        (Primitive::Map(l), Primitive::Map(r)) => l.len() == r.len() && l.iter().all(|(k, v)| r.get(k).map_or(false, |rv| primitive_eq(v, rv))),
        _ => false
    };

    return Primitive::Boolean(result);
}

pub fn neq(left: Primitive, right: Primitive) -> Primitive {
    let Primitive::Boolean(is_eq) = eq(left, right) else {
        return Primitive::Boolean(true);
    };
    Primitive::Boolean(!is_eq)
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

pub fn in_op(left: Primitive, right: Primitive) -> Primitive {
    let result = match (&left, &right) {
        (Primitive::String(needle), Primitive::String(haystack)) => haystack.contains(needle.as_str()),
        (_, Primitive::Array(arr, ..)) => arr.iter().any(|e| matches!(eq(e.clone(), left.clone()), Primitive::Boolean(true))),
        _ => false,
    };
    Primitive::Boolean(result)
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
            "in" => in_op,
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

fn compare(a: &Primitive, b: &Primitive) -> std::cmp::Ordering {
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

pub fn max(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    let mut result: Option<Primitive> = None;
    for arg in args {
        result = Some(match (result, arg) {
            (None, a) | (Some(Primitive::Null), a) => a,
            (r, Primitive::Null) => r.unwrap(),
            (Some(r), a) => if compare(&r, &a) >= std::cmp::Ordering::Equal { r } else { a },
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
            (Some(r), a) => if compare(&r, &a) <= std::cmp::Ordering::Equal { r } else { a },
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

pub fn len(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::String(s) => Primitive::Int(s.chars().count() as i64),
        Primitive::Array(a) => Primitive::Int(a.len() as i64),
        Primitive::Map(m) => Primitive::Int(m.len() as i64),
        _ => Primitive::Null,
    }
}

pub fn concat(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    let mut result = String::new();
    for arg in &args {
        if let Primitive::String(s) = arg {
            result.push_str(s);
        }
    }
    Primitive::String(result)
}

pub fn to_str(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::String(s) => Primitive::String(s.clone()),
        Primitive::Int(n) => Primitive::String(n.to_string()),
        Primitive::Float(n) => Primitive::String(n.to_string()),
        Primitive::Boolean(b) => Primitive::String(b.to_string()),
        Primitive::Null => Primitive::String("null".to_string()),
        _ => Primitive::Null,
    }
}

pub fn hex(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::Int(n) if *n >= 0 => Primitive::String(format!("0x{:x}", n)),
        Primitive::Int(n) => Primitive::String(format!("-0x{:x}", n.abs())),
        _ => Primitive::Null,
    }
}

pub fn to_upper(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::String(s) => Primitive::String(s.to_uppercase()),
        _ => Primitive::Null,
    }
}

pub fn to_lower(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::String(s) => Primitive::String(s.to_lowercase()),
        _ => Primitive::Null,
    }
}

pub fn starts_with(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    match (&args.get(0), &args.get(1)) {
        (Some(Primitive::String(s)), Some(Primitive::String(prefix))) => {
            Primitive::Boolean(s.starts_with(prefix.as_str()))
        }
        _ => Primitive::Boolean(false),
    }
}

pub fn ends_with(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    match (&args.get(0), &args.get(1)) {
        (Some(Primitive::String(s)), Some(Primitive::String(suffix))) => {
            Primitive::Boolean(s.ends_with(suffix.as_str()))
        }
        _ => Primitive::Boolean(false),
    }
}

pub fn is_empty(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    Primitive::Boolean(match &args[0] {
        Primitive::String(s) => s.is_empty() || s.trim().is_empty(),
        Primitive::Null => true,
        Primitive::Array(a) => a.is_empty(),
        Primitive::Map(m) => m.is_empty(),
        _ => false,
    })
}

pub fn is_string(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    Primitive::Boolean(matches!(args[0], Primitive::String(_)))
}

pub fn to_char_array(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::String(s) => Primitive::Array(s.chars().map(|c| Primitive::String(c.to_string())).collect()),
        _ => Primitive::Null,
    }
}

pub fn list_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    Primitive::Array(args)
}

pub fn dict_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let mut map = std::collections::HashMap::new();
    for (k, v) in kwargs {
        let key = match k {
            Primitive::String(s) => s,
            Primitive::Int(n) => n.to_string(),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Null => "null".to_string(),
            _ => continue,
        };
        map.insert(key, v);
    }
    Primitive::Map(map)
}

pub fn get_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    match (&args.get(0), &args.get(1)) {
        (Some(Primitive::Map(m)), Some(Primitive::String(key))) => {
            m.get(key).cloned().unwrap_or(Primitive::Null)
        }
        _ => Primitive::Null,
    }
}

pub fn keys_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::Map(m) => {
            let mut keys: Vec<String> = m.keys().cloned().collect();
            keys.sort();
            Primitive::Array(keys.into_iter().map(Primitive::String).collect())
        }
        _ => Primitive::Null,
    }
}

pub fn values_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::Map(m) => {
            let mut keys: Vec<String> = m.keys().cloned().collect();
            keys.sort();
            Primitive::Array(keys.into_iter().filter_map(|k| m.get(&k).cloned()).collect())
        }
        _ => Primitive::Null,
    }
}

pub fn contains_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    match (&args.get(0), &args.get(1)) {
        (Some(Primitive::Array(arr)), Some(item)) => {
            Primitive::Boolean(arr.iter().any(|e| primitive_eq(e, item)))
        }
        (Some(Primitive::String(s)), Some(Primitive::String(sub))) => {
            Primitive::Boolean(s.contains(sub.as_str()))
        }
        _ => Primitive::Boolean(false),
    }
}

pub fn set_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    let mut seen = Vec::new();
    for arg in args {
        if !seen.iter().any(|e: &Primitive| matches!(eq(e.clone(), arg.clone()), Primitive::Boolean(true))) {
            seen.push(arg);
        }
    }
    Primitive::Array(seen)
}

// --- Math functions ---

pub fn abs(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::Int(n) => Primitive::Int(n.abs()),
        Primitive::Float(n) => Primitive::Float(n.abs()),
        _ => Primitive::Null,
    }
}

pub fn sign(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::Int(n) => Primitive::Int(n.signum()),
        Primitive::Float(n) => Primitive::Int(if *n > 0.0 { 1 } else if *n < 0.0 { -1 } else { 0 }),
        _ => Primitive::Null,
    }
}

pub fn pow_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 2);
    match (&args[0], &args[1]) {
        (Primitive::Int(b), Primitive::Int(e)) => Primitive::Int(b.pow(*e as u32)),
        _ => match (as_f64(&args[0]), as_f64(&args[1])) {
            (Some(b), Some(e)) => Primitive::Float(b.powf(e)),
            _ => Primitive::Null,
        },
    }
}

pub fn round(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match as_f64(&args[0]) {
        Some(n) => Primitive::Float(bankers_round(n)),
        None => Primitive::Null,
    }
}

/// Round half to even (banker's rounding), matching Python's round().
fn bankers_round(n: f64) -> f64 {
    let i = n as i64;
    let diff = n - i as f64;
    if diff > 0.5 || diff < -0.5 {
        n.round()
    } else if diff == 0.5 || diff == -0.5 {
        // Round to even
        if i % 2 == 0 { i as f64 } else { (i + 1) as f64 }
    } else {
        i as f64
    }
}

pub fn int_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::Int(n) => Primitive::Int(*n),
        Primitive::Float(n) => Primitive::Int(*n as i64),
        Primitive::String(s) => match s.trim().parse::<i64>() {
            Ok(n) => Primitive::Int(n),
            Err(_) => match s.trim().parse::<f64>() {
                Ok(f) => Primitive::Int(f as i64),
                Err(_) => Primitive::Null,
            },
        },
        Primitive::Null => Primitive::Int(0),
        _ => Primitive::Null,
    }
}

pub fn float_fn(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    match &args[0] {
        Primitive::Int(n) => Primitive::Float(*n as f64),
        Primitive::Float(n) => Primitive::Float(*n),
        Primitive::String(s) => match s.trim().parse::<f64>() {
            Ok(f) => Primitive::Float(f),
            Err(_) => Primitive::Null,
        },
        Primitive::Null => Primitive::Float(0.0),
        _ => Primitive::Null,
    }
}

pub fn is_integer(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    Primitive::Boolean(matches!(args[0], Primitive::Int(_)))
}

pub fn is_number(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 1);
    Primitive::Boolean(matches!(args[0], Primitive::Int(_) | Primitive::Float(_)))
}

pub fn bitwise_or(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 2);
    match (&args[0], &args[1]) {
        (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a | b),
        _ => Primitive::Null,
    }
}

pub fn bitwise_and(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 2);
    match (&args[0], &args[1]) {
        (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a & b),
        _ => Primitive::Null,
    }
}

pub fn bitwise_xor(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 2);
    match (&args[0], &args[1]) {
        (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a ^ b),
        _ => Primitive::Null,
    }
}

pub fn shift_bits_left(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 2);
    match (&args[0], &args[1]) {
        (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a << b),
        _ => Primitive::Null,
    }
}

pub fn shift_bits_right(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);
    assert_eq!(args.len(), 2);
    match (&args[0], &args[1]) {
        (Primitive::Int(a), Primitive::Int(b)) => Primitive::Int(a >> b),
        _ => Primitive::Null,
    }
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
            "len" => len,
            "concat" => concat,
            "str" => to_str,
            "hex" => hex,
            "isEmpty" => is_empty,
            "isString" => is_string,
            "toCharArray" => to_char_array,
            "toUpper" => to_upper,
            "toLower" => to_lower,
            "startsWith" => starts_with,
            "endsWith" => ends_with,
            "list" => list_fn,
            "dict" => dict_fn,
            "get" => get_fn,
            "keys" => keys_fn,
            "values" => values_fn,
            "contains" => contains_fn,
            "set" => set_fn,
            "abs" => abs,
            "sign" => sign,
            "pow" => pow_fn,
            "round" => round,
            "int" => int_fn,
            "float" => float_fn,
            "isInteger" => is_integer,
            "isNumber" => is_number,
            "bitwiseOr" => bitwise_or,
            "bitwiseAnd" => bitwise_and,
            "bitwiseXor" => bitwise_xor,
            "shiftBitsLeft" => shift_bits_left,
            "shiftBitsRight" => shift_bits_right,
            _ => todo!()
        }
    }
}

pub static FUNCTIONS: Functions = Functions {};