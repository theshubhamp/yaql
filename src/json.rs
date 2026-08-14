use yaql_core::lang::Primitive;
use yaql_core::lang::primitive::{compare, type_rank};

pub fn json_to_primitive(value: &serde_json::Value) -> Primitive {
    match value {
        serde_json::Value::Null => Primitive::Null,
        serde_json::Value::Bool(b) => Primitive::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Primitive::Int(i)
            } else if let Some(f) = n.as_f64() {
                Primitive::Float(f)
            } else {
                Primitive::Null
            }
        }
        serde_json::Value::String(s) => Primitive::String(s.clone()),
        serde_json::Value::Array(arr) => {
            Primitive::Array(arr.iter().map(json_to_primitive).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj.iter() {
                map.insert(k.clone(), json_to_primitive(v));
            }
            Primitive::Map(map)
        }
    }
}

pub fn primitive_to_json(value: &Primitive) -> serde_json::Value {
    match value {
        Primitive::Null => serde_json::Value::Null,
        Primitive::Boolean(b) => serde_json::Value::Bool(*b),
        Primitive::Int(n) => serde_json::Value::Number((*n).into()),
        Primitive::Float(n) => serde_json::Value::Number(
            serde_json::Number::from_f64(*n)
                .unwrap_or_else(|| serde_json::Number::from(0)),
        ),
        Primitive::String(s) => serde_json::Value::String(s.clone()),
        Primitive::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(primitive_to_json).collect())
        }
        Primitive::Set(arr) => {
            let mut sorted: Vec<&Primitive> = arr.iter().collect();
            sorted.sort_by(|a, b| {
                match type_rank(a).cmp(&type_rank(b)) {
                    std::cmp::Ordering::Equal => compare(a, b),
                    ord => ord,
                }
            });
            serde_json::Value::Array(sorted.iter().map(|p| primitive_to_json(p)).collect())
        }
        Primitive::Map(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map.iter() {
                obj.insert(k.clone(), primitive_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        Primitive::Regex(_) => serde_json::Value::Null,
        Primitive::Lambda(_) => serde_json::Value::Null,
    }
}