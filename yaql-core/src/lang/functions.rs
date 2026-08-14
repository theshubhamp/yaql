pub use crate::lang::primitive::Primitive;
pub use crate::lang::primitive::RegexWrapper;
pub use crate::lang::primitive::LambdaBody;
pub use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EvalError(pub String);

impl EvalError {
    pub fn new(msg: impl Into<String>) -> Self {
        EvalError(msg.into())
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Function = fn(Vec<Primitive>, Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError>;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Type {
    Int,
    Float,
    Number,
    String,
    Boolean,
    Array,
    Set,
    Map,
    Regex,
    Lambda,
    Null,
    Any,
}

impl Type {
    pub fn matches(&self, val: &Primitive) -> bool {
        match (self, val) {
            (Type::Int, Primitive::Int(_)) => true,
            (Type::Float, Primitive::Float(_)) => true,
            (Type::Number, Primitive::Int(_) | Primitive::Float(_)) => true,
            (Type::String, Primitive::String(_)) => true,
            (Type::Boolean, Primitive::Boolean(_)) => true,
            (Type::Array, Primitive::Array(_)) => true,
            (Type::Set, Primitive::Set(_)) => true,
            (Type::Map, Primitive::Map(_)) => true,
            (Type::Regex, Primitive::Regex(_)) => true,
            (Type::Lambda, Primitive::Lambda(_)) => true,
            (Type::Null, Primitive::Null) => true,
            (Type::Any, _) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ArgSpec {
    Exact(usize),
    Min(usize),
    Varargs,
}

#[derive(Clone, Copy)]
pub struct Spec {
    pub name: &'static str,
    pub func: Function,
    pub args: ArgSpec,
    pub arg_types: &'static [Type],
    pub kwargs: bool,
}

impl Spec {
    pub const fn new(name: &'static str, func: Function, args: ArgSpec, arg_types: &'static [Type], kwargs: bool) -> Self {
        Spec { name, func, args, arg_types, kwargs }
    }
}

inventory::collect!(Spec);

// --- Type extraction traits ---

pub trait FromPrimitive: Sized {
    const TYPE: Type;
    fn from_primitive(p: &Primitive) -> Option<Self>;
}

impl FromPrimitive for i64 {
    const TYPE: Type = Type::Int;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::Int(n) = p { Some(*n) } else { None }
    }
}

impl FromPrimitive for f64 {
    const TYPE: Type = Type::Float;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::Float(n) = p { Some(*n) } else { None }
    }
}

impl FromPrimitive for String {
    const TYPE: Type = Type::String;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::String(s) = p { Some(s.clone()) } else { None }
    }
}

impl FromPrimitive for bool {
    const TYPE: Type = Type::Boolean;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::Boolean(b) = p { Some(*b) } else { None }
    }
}

impl FromPrimitive for Vec<Primitive> {
    const TYPE: Type = Type::Array;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::Array(a) = p { Some(a.clone()) } else { None }
    }
}

impl FromPrimitive for SetVec {
    const TYPE: Type = Type::Set;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::Set(a) = p { Some(SetVec(a.clone())) } else { None }
    }
}

impl FromPrimitive for std::collections::HashMap<String, Primitive> {
    const TYPE: Type = Type::Map;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::Map(m) = p { Some(m.clone()) } else { None }
    }
}

impl FromPrimitive for RegexWrapper {
    const TYPE: Type = Type::Regex;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::Regex(r) = p { Some(r.clone()) } else { None }
    }
}

impl FromPrimitive for crate::lang::primitive::LambdaBody {
    const TYPE: Type = Type::Lambda;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if let Primitive::Lambda(l) = p { Some(l.clone()) } else { None }
    }
}

impl<T: FromPrimitive> FromPrimitive for Option<T> {
    const TYPE: Type = T::TYPE;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if matches!(p, Primitive::Null) { Some(None) }
        else { T::from_primitive(p).map(Some) }
    }
}

pub struct Number(pub f64);
impl FromPrimitive for Number {
    const TYPE: Type = Type::Number;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        match p {
            Primitive::Int(n) => Some(Number(*n as f64)),
            Primitive::Float(n) => Some(Number(*n)),
            _ => None,
        }
    }
}

pub struct SetVec(pub Vec<Primitive>);

pub struct Any(pub Primitive);
impl FromPrimitive for Any {
    const TYPE: Type = Type::Any;
    fn from_primitive(p: &Primitive) -> Option<Self> { Some(Any(p.clone())) }
}

pub struct Null;
impl FromPrimitive for Null {
    const TYPE: Type = Type::Null;
    fn from_primitive(p: &Primitive) -> Option<Self> {
        if matches!(p, Primitive::Null) { Some(Null) } else { None }
    }
}

// --- IntoPrimitive for return ---

pub trait IntoPrimitive {
    fn into_primitive(self) -> Primitive;
}

impl IntoPrimitive for Primitive {
    fn into_primitive(self) -> Primitive { self }
}

impl IntoPrimitive for i64 {
    fn into_primitive(self) -> Primitive { Primitive::Int(self) }
}

impl IntoPrimitive for f64 {
    fn into_primitive(self) -> Primitive { Primitive::Float(self) }
}

impl IntoPrimitive for String {
    fn into_primitive(self) -> Primitive { Primitive::String(self) }
}

impl IntoPrimitive for bool {
    fn into_primitive(self) -> Primitive { Primitive::Boolean(self) }
}

impl IntoPrimitive for Null {
    fn into_primitive(self) -> Primitive { Primitive::Null }
}

impl IntoPrimitive for Vec<Primitive> {
    fn into_primitive(self) -> Primitive { Primitive::Array(self) }
}

impl IntoPrimitive for std::collections::HashMap<String, Primitive> {
    fn into_primitive(self) -> Primitive { Primitive::Map(self) }
}

impl IntoPrimitive for SetVec {
    fn into_primitive(self) -> Primitive { Primitive::Set(self.0) }
}

impl IntoPrimitive for RegexWrapper {
    fn into_primitive(self) -> Primitive { Primitive::Regex(self) }
}

impl<T: IntoPrimitive> IntoPrimitive for Option<T> {
    fn into_primitive(self) -> Primitive {
        match self {
            Some(v) => v.into_primitive(),
            None => Primitive::Null,
        }
    }
}

// --- Lookup ---

pub struct Functions;

impl Functions {
    /// Cached, pre-sorted overload lookup. Returns a `'static` slice so the
    /// per-call hot path avoids both the inventory scan and the sort.
    pub fn lookup(&self, name: &str) -> &'static [Spec] {
        cached_overloads(name)
    }
}

pub static FUNCTIONS: Functions = Functions {};

use std::sync::OnceLock;
use std::collections::HashMap as StdHashMap;

fn sort_overloads(mut overloads: Vec<Spec>) -> Vec<Spec> {
    overloads.sort_by(|a, b| {
        let score = |s: &Spec| {
            s.arg_types.iter().map(|ty| match ty {
                Type::Any => 3,
                Type::Number => 2,
                _ => 0,
            }).sum::<u32>()
        };
        let sa = score(a);
        let sb = score(b);
        sa.cmp(&sb).then_with(|| {
            b.arg_types.len().cmp(&a.arg_types.len())
        })
    });
    overloads
}

static OVERLOAD_CACHE: OnceLock<StdHashMap<String, Vec<Spec>>> = OnceLock::new();

/// Look up and pre-sort overloads for `name`, caching the result.
pub fn cached_overloads(name: &str) -> &'static Vec<Spec> {
    let map = OVERLOAD_CACHE.get_or_init(|| {
        let mut m = StdHashMap::new();
        for spec in inventory::iter::<Spec>() {
            m.entry(spec.name.to_string())
                .or_insert_with(Vec::new)
                .push(*spec);
        }
        for v in m.values_mut() {
            *v = sort_overloads(std::mem::take(v));
        }
        m
    });
    map.get(name).map(|v| v as &Vec<Spec>).unwrap_or(&EMPTY_OVERLOADS)
}

static EMPTY_OVERLOADS: Vec<Spec> = Vec::new();

// --- Dispatch ---

/// Find the best matching overload and call it.
/// `overloads` is expected to be the pre-sorted list from `FUNCTIONS.lookup`.
pub fn dispatch(overloads: &[Spec], args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
    let typed: Vec<&Spec> = overloads.iter().filter(|s| !s.arg_types.is_empty()).collect();
    let untyped: Vec<&Spec> = overloads.iter().filter(|s| s.arg_types.is_empty()).collect();
    let ordered: Vec<&Spec> = typed.into_iter().chain(untyped.into_iter()).collect();
    for spec in &ordered {
        if !spec.kwargs && !kwargs.is_empty() {
            continue;
        }
        let arg_count_ok = match spec.args {
            ArgSpec::Exact(n) => args.len() == n,
            ArgSpec::Min(n) => args.len() >= n,
            ArgSpec::Varargs => true,
        };
        if !arg_count_ok {
            continue;
        }
        let types_ok = spec.arg_types.iter().enumerate()
            .all(|(i, ty)| i >= args.len() || ty.matches(&args[i]));
        if !types_ok {
            continue;
        }
        return (spec.func)(args, kwargs);
    }
    if let Some(spec) = overloads.first() {
        if !spec.kwargs { assert_eq!(kwargs.len(), 0); }
        match spec.args {
            ArgSpec::Exact(n) => assert_eq!(args.len(), n),
            ArgSpec::Min(n) => assert!(args.len() >= n),
            ArgSpec::Varargs => {}
        }
        return (spec.func)(args, kwargs);
    }
    Ok(Primitive::Null)
}
