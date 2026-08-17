use yaql_core::lang::Primitive;
use yaql_core::lang::primitive::LambdaBody;
use yaql_core::lang::functions::{EvalError, RegexWrapper, Varargs, Kwargs, Any};
use yaql_macros::yaql_function;
use regex::{Regex, RegexBuilder};

fn build_regex(pattern: &str, ignore_case: bool) -> Option<RegexWrapper> {
    let mut builder = RegexBuilder::new(pattern);
    if ignore_case {
        builder.case_insensitive(true);
    }
    builder.build().ok().map(|re| RegexWrapper::new(re, ignore_case))
}

// regex(pattern) and regex(pattern, ignoreCase => bool)
#[yaql_function("regex")]
pub fn regex_fn(pattern: String, rest: Varargs<0>, kwargs: Kwargs) -> Result<Primitive, EvalError> {
    let ignore_case = kwargs.0.iter().find_map(|(k, v)| {
        if let Primitive::String(key) = k {
            if key == "ignoreCase" {
                if let Primitive::Boolean(b) = v { return Some(*b); }
            }
        }
        None
    }).unwrap_or(false);
    match build_regex(&pattern, ignore_case) {
        Some(r) => Ok(Primitive::Regex(r)),
        None => Ok(Primitive::Null),
    }
}
// escapeRegex(s)
#[yaql_function("escapeRegex")]
fn escape_regex(s: String) -> String {
    regex::escape(&s)
}

// isRegex(x)
#[yaql_function("isRegex")]
fn is_regex(v: Any) -> bool {
    matches!(v.0, Primitive::Regex(_))
}

// --- matches ---
// regex.matches(string) -> bool (partial match / search)
#[yaql_function("matches")]
fn regex_matches(re: RegexWrapper, s: String) -> bool {
    re.0.is_match(&s)
}

// string.matches(string) -> bool (full match via anchoring)
#[yaql_function("matches")]
fn str_matches(s: String, pattern: String) -> Option<bool> {
    let anchored = format!("^(?:{})$", pattern);
    match regex::Regex::new(&anchored) {
        Ok(re) => Some(re.is_match(&s)),
        Err(_) => None,
    }
}

// --- search ---
// regex.search(string) -> string | null  (first full match value)
// regex.search(string, selector) -> selector applied to match object
#[yaql_function("search")]
pub fn regex_search_fn(re: RegexWrapper, s: String, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let m = match re.0.find(&s) {
        Some(m) => m,
        None => return Ok(Primitive::Null),
    };
    let captures = re.0.captures(&s);
    if rest.0.is_empty() {
        return Ok(Primitive::String(m.as_str().to_string()));
    }
    // With selector — but we don't have lambda support, so only handle $ / $N / $.field
    let selector = &rest.0[0];
    apply_selector(selector, &m, captures.as_ref(), &s)
}
// --- searchAll ---
// regex.searchAll(string) -> array of strings
// regex.searchAll(string, selector) -> array of selector results
#[yaql_function("searchAll")]
pub fn regex_search_all_fn(re: RegexWrapper, s: String, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let has_selector = !rest.0.is_empty();
    let mut results = Vec::new();
    for m in re.0.find_iter(&s) {
        if !has_selector {
            results.push(Primitive::String(m.as_str().to_string()));
            continue;
        }
        let captures = re.0.captures_at(&s, m.start());
        let r = apply_selector(&rest.0[0], &m, captures.as_ref(), &s)?;
        results.push(r);
    }
    Ok(Primitive::Array(results))
}
// --- split ---
// regex.split(string) -> array
// regex.split(string, maxsplit) -> array
#[yaql_function("split")]
pub fn regex_split_fn(re: RegexWrapper, s: String, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let maxsplit = if !rest.0.is_empty() {
        if let Primitive::Int(n) = &rest.0[0] { Some(*n as usize) } else { None }
    } else { None };
    let parts: Vec<Primitive> = if let Some(n) = maxsplit {
        // Python's re.split with maxsplit: split at most n times → n+1 pieces
        split_with_max(&re.0, &s, n)
    } else {
        re.0.split(&s).map(|p| Primitive::String(p.to_string())).collect()
    };
    // If pattern has capture groups, Python inserts captured groups between splits
    if re.0.captures_len() > 1 {
        let parts = if let Some(n) = maxsplit {
            split_with_max_captures(&re.0, &s, n)
        } else {
            split_captures(&re.0, &s)
        };
        return Ok(Primitive::Array(parts));
    }
    Ok(Primitive::Array(parts))
}
// string.split(regex) -> array
#[yaql_function("split")]
fn str_split_regex_fn(s: String, re: RegexWrapper) -> Vec<Primitive> {
    if re.0.captures_len() > 1 {
        crate::regex::split_captures(&re.0, &s)
    } else {
        re.0.split(&s).map(|p| Primitive::String(p.to_string())).collect()
    }
}

// --- replace ---
// regex.replace(string, replacement) -> string
// regex.replace(string, replacement, count) -> string
#[yaql_function("replace")]
pub fn regex_replace_fn(re: RegexWrapper, s: String, replacement: String, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let count = if !rest.0.is_empty() {
        if let Primitive::Int(n) = &rest.0[0] { Some(*n as usize) } else { None }
    } else { None };
    let result = apply_backrefs(&re.0, &s, &replacement, count);
    Ok(Primitive::String(result))
}
// string.replace(regex, replacement) -> string
// string.replace(regex, replacement, count) -> string
#[yaql_function("replace")]
pub fn str_replace_regex_fn(s: String, re: RegexWrapper, replacement: String, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let count = if !rest.0.is_empty() {
        if let Primitive::Int(n) = &rest.0[0] { Some(*n as usize) } else { None }
    } else { None };
    let result = apply_backrefs(&re.0, &s, &replacement, count);
    Ok(Primitive::String(result))
}
// --- replaceBy (without lambda, only string replacement) ---
// regex.replaceBy(string, replacement) -> string
// regex.replaceBy(string, replacement, count) -> string
#[yaql_function("replaceBy")]
pub fn regex_replace_by_fn(re: RegexWrapper, s: String, replacement: String, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let count = if !rest.0.is_empty() {
        if let Primitive::Int(n) = &rest.0[0] { Some(*n as usize) } else { None }
    } else { None };
    let result = apply_backrefs(&re.0, &s, &replacement, count);
    Ok(Primitive::String(result))
}
#[yaql_function("replaceBy")]
pub fn str_replace_by_regex_fn(s: String, re: RegexWrapper, replacement: String, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let count = if !rest.0.is_empty() {
        if let Primitive::Int(n) = &rest.0[0] { Some(*n as usize) } else { None }
    } else { None };
    let result = apply_backrefs(&re.0, &s, &replacement, count);
    Ok(Primitive::String(result))
}
// --- replaceBy (lambda: replacement is a lambda called with match object) ---
#[yaql_function("replaceBy")]
pub fn regex_replace_by_lambda_fn(re: RegexWrapper, s: String, lambda: LambdaBody, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let count = if !rest.0.is_empty() {
        if let Primitive::Int(n) = &rest.0[0] { Some(*n as usize) } else { None }
    } else { None };
    let mut result = String::new();
    let mut last_end = 0;
    let mut matched = 0;
    for cap in re.0.captures_iter(&s) {
        if let Some(c) = count { if matched >= c { break; } }
        let m = cap.get(0).unwrap();
        result.push_str(&s[last_end..m.start()]);
        let match_obj = match_to_map(&m, &s);
        let groups: Vec<Primitive> = (0..cap.len()).map(|i| {
            cap.get(i).map(|g| match_to_map(&g, &s)).unwrap_or(Primitive::Null)
        }).collect();
        let mut interp = yaql_core::interpreter::Interpreter { contexts: (*lambda.env).clone(), current_func: None };
        interp.push_context(Primitive::Array(groups));
        interp.push_context(match_obj);
        let replacement = yaql_core::interpreter::eval_body(&mut interp, &lambda.body)?;
        if let Primitive::String(r) = replacement { result.push_str(&r); }
        last_end = m.end();
        matched += 1;
    }
    result.push_str(&s[last_end..]);
    Ok(Primitive::String(result))
}
#[yaql_function("replaceBy")]
pub fn str_replace_by_lambda_fn(s: String, re: RegexWrapper, lambda: LambdaBody, rest: Varargs<0>) -> Result<Primitive, EvalError> {
    let count = if !rest.0.is_empty() {
        if let Primitive::Int(n) = &rest.0[0] { Some(*n as usize) } else { None }
    } else { None };
    let mut result = String::new();
    let mut last_end = 0;
    let mut matched = 0;
    for cap in re.0.captures_iter(&s) {
        if let Some(c) = count { if matched >= c { break; } }
        let m = cap.get(0).unwrap();
        result.push_str(&s[last_end..m.start()]);
        let match_obj = match_to_map(&m, &s);
        let groups: Vec<Primitive> = (0..cap.len()).map(|i| {
            cap.get(i).map(|g| match_to_map(&g, &s)).unwrap_or(Primitive::Null)
        }).collect();
        let mut interp = yaql_core::interpreter::Interpreter { contexts: (*lambda.env).clone(), current_func: None };
        interp.push_context(Primitive::Array(groups));
        interp.push_context(match_obj);
        let replacement = yaql_core::interpreter::eval_body(&mut interp, &lambda.body)?;
        if let Primitive::String(r) = replacement { result.push_str(&r); }
        last_end = m.end();
        matched += 1;
    }
    result.push_str(&s[last_end..]);
    Ok(Primitive::String(result))
}
fn match_to_map(m: &regex::Match, s: &str) -> Primitive {
    let mut map = std::collections::HashMap::new();
    map.insert("value".to_string(), Primitive::String(m.as_str().to_string()));
    map.insert("start".to_string(), Primitive::Int(m.start() as i64));
    map.insert("end".to_string(), Primitive::Int(m.end() as i64));
    Primitive::Map(map)
}

fn capture_to_map(cap: &regex::Captures, idx: usize, s: &str) -> Primitive {
    match cap.get(idx) {
        Some(m) => match_to_map(&m, s),
        None => Primitive::Null,
    }
}

fn apply_selector(selector: &Primitive, m: &regex::Match, captures: Option<&regex::Captures>, s: &str) -> Result<Primitive, yaql_core::lang::functions::EvalError> {
    use yaql_core::interpreter::Interpreter;
    let Primitive::Lambda(lambda) = selector else {
        return Ok(Primitive::String(m.as_str().to_string()));
    };
    // Build context: push match object as $, and capture groups array for $1/$2
    let match_obj = match_to_map(m, s);
    let mut interp = Interpreter { contexts: (*lambda.env).clone(), current_func: None };
    // For $1/$2: push array of capture group objects
    if let Some(cap) = captures {
        let groups: Vec<Primitive> = (0..cap.len()).map(|i| {
            cap.get(i).map(|g| match_to_map(&g, s)).unwrap_or(Primitive::Null)
        }).collect();
        interp.push_context(Primitive::Array(groups));
    }
    // Push match object as $ (top of stack)
    interp.push_context(match_obj);
    yaql_core::interpreter::eval_body(&mut interp, &lambda.body)
}

fn apply_backrefs(re: &Regex, s: &str, replacement: &str, count: Option<usize>) -> String {
    // Handle backreferences: \1, \2, ... and ${name}
    let captures_iter: Vec<regex::Captures> = if let Some(n) = count {
        re.captures_iter(&s).take(n).collect()
    } else {
        re.captures_iter(&s).collect()
    };

    if captures_iter.is_empty() {
        return s.to_string();
    }

    let mut result = String::new();
    let mut last_end = 0;
    for cap in &captures_iter {
        let m = cap.get(0).unwrap();
        result.push_str(&s[last_end..m.start()]);
        result.push_str(&expand_backrefs(cap, replacement));
        last_end = m.end();
    }
    result.push_str(&s[last_end..]);
    result
}

fn expand_backrefs(cap: &regex::Captures, replacement: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = replacement.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next.is_ascii_digit() {
                let n: usize = next.to_digit(10).unwrap() as usize;
                if n <= cap.len() {
                    if let Some(m) = cap.get(n) {
                        result.push_str(m.as_str());
                    }
                }
                i += 2;
                continue;
            } else if next == '\\' {
                result.push('\\');
                i += 2;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn split_with_max(re: &Regex, s: &str, maxsplit: usize) -> Vec<Primitive> {
    let mut result = Vec::new();
    let mut last_end = 0;
    let mut splits = 0;
    for m in re.find_iter(&s) {
        if splits >= maxsplit {
            break;
        }
        result.push(Primitive::String(s[last_end..m.start()].to_string()));
        last_end = m.end();
        splits += 1;
    }
    result.push(Primitive::String(s[last_end..].to_string()));
    result
}

pub(crate) fn split_captures(re: &Regex, s: &str) -> Vec<Primitive> {
    let mut result = Vec::new();
    let mut last_end = 0;
    for cap in re.captures_iter(&s) {
        let m = cap.get(0).unwrap();
        result.push(Primitive::String(s[last_end..m.start()].to_string()));
        // Insert capture groups (1..n)
        for i in 1..cap.len() {
            if let Some(g) = cap.get(i) {
                result.push(Primitive::String(g.as_str().to_string()));
            } else {
                result.push(Primitive::Null);
            }
        }
        last_end = m.end();
    }
    result.push(Primitive::String(s[last_end..].to_string()));
    result
}

fn split_with_max_captures(re: &Regex, s: &str, maxsplit: usize) -> Vec<Primitive> {
    let mut result = Vec::new();
    let mut last_end = 0;
    let mut splits = 0;
    for cap in re.captures_iter(&s) {
        if splits >= maxsplit {
            break;
        }
        let m = cap.get(0).unwrap();
        result.push(Primitive::String(s[last_end..m.start()].to_string()));
        for i in 1..cap.len() {
            if let Some(g) = cap.get(i) {
                result.push(Primitive::String(g.as_str().to_string()));
            } else {
                result.push(Primitive::Null);
            }
        }
        last_end = m.end();
        splits += 1;
    }
    result.push(Primitive::String(s[last_end..].to_string()));
    result
}