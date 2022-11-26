use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Primitive {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Primitive>),
    Map(HashMap<Primitive, Primitive>),
    Null,
}

pub fn truthy(value: &Primitive) -> bool {
    return match value {
        Primitive::String(string) => string.len() > 0,
        Primitive::Number(num) => *num != 0.0,
        Primitive::Boolean(bool) => *bool,
        Primitive::Array(vec) => vec.len() > 0,
        Primitive::Map(map) => map.len() > 0,
        Primitive::Null => false,
    }
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
        (Primitive::String(left_string), Primitive::String(right_string)) => left_string == right_string,
        (Primitive::Number(left_num), Primitive::Number(right_num)) => left_num == right_num,
        (Primitive::Boolean(left_bool), Primitive::Boolean(right_bool)) => left_bool == right_bool,
        _ => false
    };

    return Primitive::Boolean(result);
}

pub fn neq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (left, right) {
        (Primitive::String(left_string), Primitive::String(right_string)) => left_string != right_string,
        (Primitive::Number(left_num), Primitive::Number(right_num)) => left_num != right_num,
        (Primitive::Boolean(left_bool), Primitive::Boolean(right_bool)) => left_bool != right_bool,
        _ => false
    };

    return Primitive::Boolean(result);
}

pub fn lt(left: Primitive, right: Primitive) -> Primitive {
    let result = match (left, right) {
        (Primitive::Number(left_num), Primitive::Number(right_num)) => left_num < right_num,
        _ => false
    };

    return Primitive::Boolean(result);
}

pub fn lteq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (left, right) {
        (Primitive::Number(left_num), Primitive::Number(right_num)) => left_num <= right_num,
        _ => false
    };

    return Primitive::Boolean(result);
}

pub fn gt(left: Primitive, right: Primitive) -> Primitive {
    let result = match (left, right) {
        (Primitive::Number(left_num), Primitive::Number(right_num)) => left_num > right_num,
        _ => false
    };

    return Primitive::Boolean(result);
}

pub fn gteq(left: Primitive, right: Primitive) -> Primitive {
    let result = match (left, right) {
        (Primitive::Number(left_num), Primitive::Number(right_num)) => left_num >= right_num,
        _ => false
    };

    return Primitive::Boolean(result);
}

pub struct BinaryOperators;

impl BinaryOperators {
    pub fn lookup(&self, name: String) -> fn(Primitive, Primitive) -> Primitive {
        return match name.as_str() {
            "and" => and,
            "or" => or,
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
            return Primitive::Number(index as f64);
        }

        index += 1;
    }

    return Primitive::Number(index as f64);
}

pub fn select_all_cases(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    assert_eq!(kwargs.len(), 0);

    let mut cases = Vec::new();
    for predicate in args {
        if truthy(&predicate) {
            cases.push(predicate)
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

pub struct Functions;

impl Functions {
    pub fn lookup(&self, name: String) -> fn(Vec<Primitive>, Vec<(Primitive, Primitive)>) -> Primitive {
        return match name.as_str() {
            "switch" => switch,
            "selectCase" => select_case,
            "selectAllCases" => select_all_cases,
            "examine" => examine,
            _ => todo!()
        }
    }
}

pub static FUNCTIONS: Functions = Functions {};
