use crate::lang::primitive::Primitive;
use crate::lang::functions::{FromPrimitive, IntoPrimitive, Any, Null, Spec};
use crate::yaql_function;

yaql_function!("len", len_string(s: String) -> i64 { s.chars().count() as i64 });
yaql_function!("len", len_array(a: Vec<Primitive>) -> i64 { a.len() as i64 });
yaql_function!("len", len_set(a: SetVec) -> i64 { a.0.len() as i64 });
yaql_function!("len", len_map(m: std::collections::HashMap<String, Primitive>) -> i64 { m.len() as i64 });

yaql_function!("str", str_from_string(s: String) -> String { s });
yaql_function!("str", str_from_int(n: i64) -> String { n.to_string() });
yaql_function!("str", str_from_float(n: f64) -> String { n.to_string() });
yaql_function!("str", str_from_boolean(b: bool) -> String { b.to_string() });
yaql_function!("str", str_from_null(_n: Null) -> String { "null".to_string() });

yaql_function!("hex", hex(n: i64) -> String { if n >= 0 { format!("0x{:x}", n) } else { format!("-0x{:x}", n.abs()) } });
yaql_function!("toUpper", to_upper(s: String) -> String { s.to_uppercase() });
yaql_function!("toLower", to_lower(s: String) -> String { s.to_lowercase() });
yaql_function!("startsWith", starts_with(s: String, prefix: String) -> bool { s.starts_with(prefix.as_str()) });
yaql_function!("endsWith", ends_with(s: String, suffix: String) -> bool { s.ends_with(suffix.as_str()) });

yaql_function!("isEmpty", is_empty_string(s: String) -> bool { s.is_empty() || s.trim().is_empty() });
yaql_function!("isEmpty", is_empty_array(a: Vec<Primitive>) -> bool { a.is_empty() });
yaql_function!("isEmpty", is_empty_map(m: std::collections::HashMap<String, Primitive>) -> bool { m.is_empty() });
yaql_function!("isEmpty", is_empty_null(_n: Null) -> bool { true });

yaql_function!("isString", is_string(v: Any) -> bool { matches!(v.0, Primitive::String(_)) });
yaql_function!("toCharArray", to_char_array(s: String) -> Vec<Primitive> {
    s.chars().map(|c| Primitive::String(c.to_string())).collect()
});