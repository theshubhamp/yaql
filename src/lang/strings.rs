use crate::lang::primitive::Primitive;
use crate::lang::functions::{FromPrimitive, IntoPrimitive, Any, Null, Spec, SetVec};
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;
use crate::lang::functions::Type;

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

// --- split ---

pub fn split_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Null };
    if args.len() == 1 {
        return Primitive::Array(s.split_whitespace().map(|p| Primitive::String(p.to_string())).collect());
    }
    let Primitive::String(sep) = &args[1] else { return Primitive::Null };
    if sep.is_empty() {
        return Primitive::Array(s.chars().map(|c| Primitive::String(c.to_string())).collect());
    }
    Primitive::Array(s.split(sep.as_str()).map(|p| Primitive::String(p.to_string())).collect())
}
yaql_raw_function!("split", split_fn, ArgSpec::Min(1), [Type::String], false);

pub fn right_split_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Null };
    let Primitive::String(sep) = &args[1] else { return Primitive::Null };
    if sep.is_empty() {
        return Primitive::Array(s.chars().map(|c| Primitive::String(c.to_string())).collect());
    }
    let count = if args.len() > 2 {
        if let Primitive::Int(n) = &args[2] { *n as usize } else { 0 }
    } else { 0 };
    let parts: Vec<&str> = s.rsplitn(count + 1, sep.as_str()).collect();
    Primitive::Array(parts.into_iter().rev().map(|p| Primitive::String(p.to_string())).collect())
}
yaql_raw_function!("rightSplit", right_split_fn, ArgSpec::Min(2), [Type::String, Type::String], false);

// --- join ---

fn primitive_to_str(p: &Primitive) -> String {
    match p {
        Primitive::String(s) => s.clone(),
        Primitive::Int(n) => n.to_string(),
        Primitive::Float(n) => n.to_string(),
        Primitive::Boolean(b) => b.to_string(),
        Primitive::Null => "null".to_string(),
        _ => String::new(),
    }
}

pub fn join_method(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::Array(arr) = &args[0] else { return Primitive::Null };
    let Primitive::String(sep) = &args[1] else { return Primitive::Null };
    let parts: Vec<String> = arr.iter().map(primitive_to_str).collect();
    Primitive::String(parts.join(sep))
}
yaql_raw_function!("join", join_method, ArgSpec::Exact(2), [Type::Array, Type::String], false);

pub fn join_pythonic(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(sep) = &args[0] else { return Primitive::Null };
    let Primitive::Array(arr) = &args[1] else { return Primitive::Null };
    let parts: Vec<String> = arr.iter().map(primitive_to_str).collect();
    Primitive::String(parts.join(sep))
}
yaql_raw_function!("join", join_pythonic, ArgSpec::Exact(2), [Type::String, Type::Array], false);

// --- trim ---

yaql_function!("trim", trim_default(s: String) -> String { s.trim().to_string() });
yaql_function!("trim", trim_chars(s: String, chars: String) -> String {
    s.trim_matches(|c| chars.contains(c)).to_string()
});

yaql_function!("trimLeft", trim_left_default(s: String) -> String { s.trim_start().to_string() });
yaql_function!("trimLeft", trim_left_chars(s: String, chars: String) -> String {
    s.trim_start_matches(|c| chars.contains(c)).to_string()
});

yaql_function!("trimRight", trim_right_default(s: String) -> String { s.trim_end().to_string() });
yaql_function!("trimRight", trim_right_chars(s: String, chars: String) -> String {
    s.trim_end_matches(|c| chars.contains(c)).to_string()
});

// --- norm ---

yaql_function!("norm", norm_str(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
});
yaql_function!("norm", norm_null(_n: Null) -> Null { Null });

// --- replace ---

pub fn replace_str(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Null };
    let Primitive::String(old) = &args[1] else { return Primitive::Null };
    let Primitive::String(new) = &args[2] else { return Primitive::Null };
    if old.is_empty() { return Primitive::String(s.clone()); }
    let result = if args.len() > 3 {
        let Primitive::Int(count) = args[3] else { return Primitive::String(s.clone()); };
        s.replacen(old.as_str(), new.as_str(), count as usize)
    } else {
        s.replace(old.as_str(), new.as_str())
    };
    Primitive::String(result)
}
yaql_raw_function!("replace", replace_str, ArgSpec::Min(3), [Type::String, Type::String, Type::String], false);

pub fn replace_dict(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Null };
    let count = if args.len() > 2 {
        if let Primitive::Int(c) = &args[2] { Some(*c) } else { None }
    } else { None };

    let dict = if let Some(Primitive::Map(m)) = args.get(1) {
        Some(m.clone())
    } else if kwargs.iter().any(|(k, _)| matches!(k, Primitive::String(_))) {
        let mut map = std::collections::HashMap::new();
        for (k, v) in &kwargs {
            if let Primitive::String(key) = k {
                map.insert(key.clone(), v.clone());
            }
        }
        Some(map)
    } else { None };

    if dict.is_none() {
        return replace_str(args, kwargs);
    }

    let mut result = s.clone();
    if let Some(dict) = dict {
        for (key_str, v) in &dict {
            let val_str = primitive_to_str(v);
            if key_str.is_empty() { continue; }
            result = if let Some(c) = count {
                result.replacen(key_str, &val_str, c as usize)
            } else {
                result.replace(key_str, &val_str)
            };
        }
    }
    Primitive::String(result)
}
yaql_raw_function!("replace", replace_dict, ArgSpec::Min(1), [Type::String], true);

// --- substring ---

pub fn substring_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Null };
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = match &args[1] {
        Primitive::Int(n) => if *n < 0 { (len + n).max(0) as usize } else { (*n as usize).min(len as usize) },
        _ => return Primitive::Null,
    };
    let end = if args.len() > 2 {
        match &args[2] {
            Primitive::Int(n) => {
                if *n < 0 { (len + n + 1).max(0) as usize }
                else { (start as i64 + n) as usize }
            }
            _ => return Primitive::Null,
        }
    } else {
        len as usize
    };
    let end = end.min(len as usize);
    if end < start { return Primitive::String(String::new()); }
    Primitive::String(chars[start..end].iter().collect())
}
yaql_raw_function!("substring", substring_fn, ArgSpec::Min(2), [Type::String, Type::Int], false);

// --- indexOf / lastIndexOf ---

pub fn index_of_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Null };
    let Primitive::String(needle) = &args[1] else { return Primitive::Null };
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = if args.len() > 2 {
        match &args[2] { Primitive::Int(n) => if *n < 0 { (len + n).max(0) as usize } else { (*n as usize).min(len as usize) }, _ => 0 }
    } else { 0 };
    let end = if args.len() > 3 {
        match &args[3] {
            Primitive::Int(n) => {
                let e = if *n < 0 { (len + n + 1).max(0) as usize }
                        else if (*n as usize) < start { (start + *n as usize).min(len as usize) }
                        else { (*n as usize).min(len as usize) };
                e
            }
            _ => len as usize,
        }
    } else { len as usize };
    if end < start { return Primitive::Int(-1); }
    let hay: String = chars[start..end].iter().collect();
    match hay.find(needle.as_str()) {
        Some(pos) => {
            let char_pos = hay[..pos].chars().count();
            Primitive::Int((start + char_pos) as i64)
        }
        None => Primitive::Int(-1),
    }
}
yaql_raw_function!("indexOf", index_of_fn, ArgSpec::Min(2), [Type::String, Type::String], false);

pub fn last_index_of_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Null };
    let Primitive::String(needle) = &args[1] else { return Primitive::Null };
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = if args.len() > 2 {
        match &args[2] { Primitive::Int(n) => if *n < 0 { (len + n).max(0) as usize } else { (*n as usize).min(len as usize) }, _ => 0 }
    } else { 0 };
    let end = if args.len() > 3 {
        match &args[3] {
            Primitive::Int(n) => {
                let e = if *n < 0 { (len + n + 1).max(0) as usize }
                        else if (*n as usize) < start { (start + *n as usize).min(len as usize) }
                        else { (*n as usize).min(len as usize) };
                e
            }
            _ => len as usize,
        }
    } else { len as usize };
    if end < start { return Primitive::Int(-1); }
    let hay: String = chars[start..end].iter().collect();
    match hay.rfind(needle.as_str()) {
        Some(pos) => {
            let char_pos = hay[..pos].chars().count();
            Primitive::Int((start + char_pos) as i64)
        }
        None => Primitive::Int(-1),
    }
}
yaql_raw_function!("lastIndexOf", last_index_of_fn, ArgSpec::Min(2), [Type::String, Type::String], false);

// --- startsWith / endsWith with varargs ---

pub fn starts_with_varargs(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Boolean(false) };
    let result = args[1..].iter().any(|p| {
        if let Primitive::String(prefix) = p { s.starts_with(prefix.as_str()) } else { false }
    });
    Primitive::Boolean(result)
}
yaql_raw_function!("startsWith", starts_with_varargs, ArgSpec::Min(2), [Type::String], false);

pub fn ends_with_varargs(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let Primitive::String(s) = &args[0] else { return Primitive::Boolean(false) };
    let result = args[1..].iter().any(|p| {
        if let Primitive::String(suffix) = p { s.ends_with(suffix.as_str()) } else { false }
    });
    Primitive::Boolean(result)
}
yaql_raw_function!("endsWith", ends_with_varargs, ArgSpec::Min(2), [Type::String], false);