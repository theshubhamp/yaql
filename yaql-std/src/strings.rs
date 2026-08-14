use yaql_core::lang::Primitive;
use yaql_core::lang::functions::EvalError;
use yaql_core::lang::functions::ArgSpec;
use yaql_core::lang::functions::Type;
use yaql_macros::yaql_function;
use crate::yaql_raw_function;

#[yaql_function("len")]
fn len_string(s: String) -> i64 { s.chars().count() as i64 }

#[yaql_function("len")]
fn len_array(a: Vec<Primitive>) -> i64 { a.len() as i64 }

#[yaql_function("len")]
fn len_set(a: SetVec) -> i64 { a.0.len() as i64 }

#[yaql_function("len")]
fn len_map(m: std::collections::HashMap<String, Primitive>) -> i64 { m.len() as i64 }

#[yaql_function("str")]
fn str_from_string(s: String) -> String { s }

#[yaql_function("str")]
fn str_from_int(n: i64) -> String { n.to_string() }

#[yaql_function("str")]
fn str_from_float(n: f64) -> String { n.to_string() }

#[yaql_function("str")]
fn str_from_boolean(b: bool) -> String { b.to_string() }

#[yaql_function("str")]
fn str_from_null(_n: Null) -> String { "null".to_string() }

#[yaql_function("hex")]
fn hex(n: i64) -> String { if n >= 0 { format!("0x{:x}", n) } else { format!("-0x{:x}", n.abs()) } }

#[yaql_function("toUpper")]
fn to_upper(s: String) -> String { s.to_uppercase() }

#[yaql_function("toLower")]
fn to_lower(s: String) -> String { s.to_lowercase() }

#[yaql_function("startsWith")]
fn starts_with(s: String, prefix: String) -> bool { s.starts_with(prefix.as_str()) }

#[yaql_function("endsWith")]
fn ends_with(s: String, suffix: String) -> bool { s.ends_with(suffix.as_str()) }

#[yaql_function("isEmpty")]
fn is_empty_string(s: String) -> bool { s.is_empty() || s.trim().is_empty() }

#[yaql_function("isEmpty")]
fn is_empty_array(a: Vec<Primitive>) -> bool { a.is_empty() }

#[yaql_function("isEmpty")]
fn is_empty_map(m: std::collections::HashMap<String, Primitive>) -> bool { m.is_empty() }

#[yaql_function("isEmpty")]
fn is_empty_null(_n: Null) -> bool { true }

#[yaql_function("isString")]
fn is_string(v: Any) -> bool { matches!(v.0, Primitive::String(_)) }

#[yaql_function("toCharArray")]
fn to_char_array(s: String) -> Vec<Primitive> {
    s.chars().map(|c| Primitive::String(c.to_string())).collect()
}

// --- split ---

#[yaql_function("split")]
fn split_whitespace(s: String) -> Vec<Primitive> {
    s.split_whitespace().map(|p| Primitive::String(p.to_string())).collect()
}

#[yaql_function("split")]
fn split_delim(s: String, sep: String) -> Vec<Primitive> {
    if sep.is_empty() {
        s.chars().map(|c| Primitive::String(c.to_string())).collect()
    } else {
        s.split(sep.as_str()).map(|p| Primitive::String(p.to_string())).collect()
    }
}

#[yaql_function("rightSplit")]
fn right_split_2(s: String, sep: String) -> Vec<Primitive> {
    if sep.is_empty() {
        s.chars().map(|c| Primitive::String(c.to_string())).collect()
    } else {
        let parts: Vec<&str> = s.rsplitn(1, sep.as_str()).collect();
        parts.into_iter().rev().map(|p| Primitive::String(p.to_string())).collect()
    }
}

#[yaql_function("rightSplit")]
fn right_split_3(s: String, sep: String, count: i64) -> Vec<Primitive> {
    if sep.is_empty() {
        s.chars().map(|c| Primitive::String(c.to_string())).collect()
    } else {
        let parts: Vec<&str> = s.rsplitn((count as usize) + 1, sep.as_str()).collect();
        parts.into_iter().rev().map(|p| Primitive::String(p.to_string())).collect()
    }
}

// --- join ---

pub(crate) fn primitive_to_str(p: &Primitive) -> String {
    match p {
        Primitive::String(s) => s.clone(),
        Primitive::Int(n) => n.to_string(),
        Primitive::Float(n) => n.to_string(),
        Primitive::Boolean(b) => b.to_string(),
        Primitive::Null => "null".to_string(),
        _ => String::new(),
    }
}

#[yaql_function("join")]
fn join_method(arr: Vec<Primitive>, sep: String) -> String {
    let parts: Vec<String> = arr.iter().map(crate::strings::primitive_to_str).collect();
    parts.join(sep.as_str())
}

#[yaql_function("join")]
fn join_pythonic(sep: String, arr: Vec<Primitive>) -> String {
    let parts: Vec<String> = arr.iter().map(crate::strings::primitive_to_str).collect();
    parts.join(sep.as_str())
}

// --- trim ---

#[yaql_function("trim")]
fn trim_default(s: String) -> String { s.trim().to_string() }

#[yaql_function("trim")]
fn trim_chars(s: String, chars: String) -> String {
    s.trim_matches(|c| chars.contains(c)).to_string()
}

#[yaql_function("trimLeft")]
fn trim_left_default(s: String) -> String { s.trim_start().to_string() }

#[yaql_function("trimLeft")]
fn trim_left_chars(s: String, chars: String) -> String {
    s.trim_start_matches(|c| chars.contains(c)).to_string()
}

#[yaql_function("trimRight")]
fn trim_right_default(s: String) -> String { s.trim_end().to_string() }

#[yaql_function("trimRight")]
fn trim_right_chars(s: String, chars: String) -> String {
    s.trim_end_matches(|c| chars.contains(c)).to_string()
}

// --- norm ---

#[yaql_function("norm")]
fn norm_str(s: String) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

#[yaql_function("norm")]
fn norm_null(_n: Null) -> Null { Null }

// --- replace ---

#[yaql_function("replace")]
fn replace_str_3(s: String, old: String, new: String) -> String {
    if old.is_empty() { s } else { s.replace(old.as_str(), new.as_str()) }
}

#[yaql_function("replace")]
fn replace_str_4(s: String, old: String, new: String, count: i64) -> String {
    if old.is_empty() { s } else { s.replacen(old.as_str(), new.as_str(), count as usize) }
}

pub fn replace_dict(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let Primitive::String(s) = &args[0] else { return Ok(Primitive::Null) };
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
        let Primitive::String(old) = &args[1] else { return Ok(Primitive::String(s.clone())); };
        let Primitive::String(new) = &args[2] else { return Ok(Primitive::String(s.clone())); };
        if old.is_empty() { return Ok(Primitive::String(s.clone())); }
        let result = if let Some(c) = count {
            s.replacen(old.as_str(), new.as_str(), c as usize)
        } else {
            s.replace(old.as_str(), new.as_str())
        };
        return Ok(Primitive::String(result));
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
    Ok(Primitive::String(result))
}
yaql_raw_function!("replace", replace_dict, ArgSpec::Min(1), [Type::String], true);

// --- substring ---

#[yaql_function("substring")]
fn substring_2(s: String, start: i64) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
    chars[start..].iter().collect()
}

#[yaql_function("substring")]
fn substring_3(s: String, start: i64, length: i64) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) };
    let end = if length < 0 { (len + length + 1).max(0) as usize } else { (start as i64 + length) as usize };
    let end = end.min(len as usize);
    if end < start { String::new() } else { chars[start..end].iter().collect() }
}

// --- indexOf / lastIndexOf ---

pub(crate) fn norm_start(len: i64, start: i64) -> usize {
    if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(len as usize) }
}

pub(crate) fn norm_end(len: i64, start: usize, end: i64) -> usize {
    if end < 0 { (len + end + 1).max(0) as usize }
    else if (end as usize) < start { (start + end as usize).min(len as usize) }
    else { (end as usize).min(len as usize) }
}

#[yaql_function("indexOf")]
fn index_of_2(s: String, needle: String) -> i64 {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = 0usize;
    let end = len as usize;
    let hay: String = chars[start..end].iter().collect();
    match hay.find(needle.as_str()) {
        Some(pos) => { let char_pos = hay[..pos].chars().count(); (start + char_pos) as i64 }
        None => -1,
    }
}

#[yaql_function("indexOf")]
fn index_of_3(s: String, needle: String, start: i64) -> i64 {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = crate::strings::norm_start(len, start);
    let end = len as usize;
    if end < start { -1 } else {
        let hay: String = chars[start..end].iter().collect();
        match hay.find(needle.as_str()) {
            Some(pos) => { let char_pos = hay[..pos].chars().count(); (start + char_pos) as i64 }
            None => -1,
        }
    }
}

#[yaql_function("indexOf")]
fn index_of_4(s: String, needle: String, start: i64, end: i64) -> i64 {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = crate::strings::norm_start(len, start);
    let end = crate::strings::norm_end(len, start, end);
    if end < start { -1 } else {
        let hay: String = chars[start..end].iter().collect();
        match hay.find(needle.as_str()) {
            Some(pos) => { let char_pos = hay[..pos].chars().count(); (start + char_pos) as i64 }
            None => -1,
        }
    }
}

#[yaql_function("lastIndexOf")]
fn last_index_of_2(s: String, needle: String) -> i64 {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = 0usize;
    let end = len as usize;
    let hay: String = chars[start..end].iter().collect();
    match hay.rfind(needle.as_str()) {
        Some(pos) => { let char_pos = hay[..pos].chars().count(); (start + char_pos) as i64 }
        None => -1,
    }
}

#[yaql_function("lastIndexOf")]
fn last_index_of_3(s: String, needle: String, start: i64) -> i64 {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = crate::strings::norm_start(len, start);
    let end = len as usize;
    if end < start { -1 } else {
        let hay: String = chars[start..end].iter().collect();
        match hay.rfind(needle.as_str()) {
            Some(pos) => { let char_pos = hay[..pos].chars().count(); (start + char_pos) as i64 }
            None => -1,
        }
    }
}

#[yaql_function("lastIndexOf")]
fn last_index_of_4(s: String, needle: String, start: i64, end: i64) -> i64 {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = crate::strings::norm_start(len, start);
    let end = crate::strings::norm_end(len, start, end);
    if end < start { -1 } else {
        let hay: String = chars[start..end].iter().collect();
        match hay.rfind(needle.as_str()) {
            Some(pos) => { let char_pos = hay[..pos].chars().count(); (start + char_pos) as i64 }
            None => -1,
        }
    }
}

// --- startsWith / endsWith with varargs ---

pub fn starts_with_varargs(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let Primitive::String(s) = &args[0] else { return Ok(Primitive::Boolean(false)) };
    let result = args[1..].iter().any(|p| {
        if let Primitive::String(prefix) = p { s.starts_with(prefix.as_str()) } else { false }
    });
    Ok(Primitive::Boolean(result))
}
yaql_raw_function!("startsWith", starts_with_varargs, ArgSpec::Min(2), [Type::String], false);

pub fn ends_with_varargs(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let Primitive::String(s) = &args[0] else { return Ok(Primitive::Boolean(false)) };
    let result = args[1..].iter().any(|p| {
        if let Primitive::String(suffix) = p { s.ends_with(suffix.as_str()) } else { false }
    });
    Ok(Primitive::Boolean(result))
}
yaql_raw_function!("endsWith", ends_with_varargs, ArgSpec::Min(2), [Type::String], false);
