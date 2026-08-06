pub use crate::lang::primitive::Primitive;
pub use crate::lang::primitive::RegexWrapper;
pub use std::collections::HashMap;

pub type Function = fn(Vec<Primitive>, Vec<(Primitive, Primitive)>) -> Primitive;

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

// --- Macros ---

#[macro_export]
macro_rules! yaql_function {
    ($yaql_name:literal, $name:ident() -> $ret:ty $body:block) => {
        mod $name {
            use crate::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(0), &[], false);
            pub fn func(_args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
                let __ret: $ret = $body;
                IntoPrimitive::into_primitive(__ret)
            }
        }
        inventory::submit! { $name::SPEC }
    };
    ($yaql_name:literal, $name:ident($p0:ident : $t0:ty) -> $ret:ty $body:block) => {
        mod $name {
            use crate::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(1), &[<$t0 as FromPrimitive>::TYPE], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
                let $p0 = match <$t0 as FromPrimitive>::from_primitive(&args[0]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let __ret: $ret = $body;
                IntoPrimitive::into_primitive(__ret)
            }
        }
        inventory::submit! { $name::SPEC }
    };
    ($yaql_name:literal, $name:ident($p0:ident : $t0:ty, $p1:ident : $t1:ty) -> $ret:ty $body:block) => {
        mod $name {
            use crate::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(2), &[<$t0 as FromPrimitive>::TYPE, <$t1 as FromPrimitive>::TYPE], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
                let $p0 = match <$t0 as FromPrimitive>::from_primitive(&args[0]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let $p1 = match <$t1 as FromPrimitive>::from_primitive(&args[1]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let __ret: $ret = $body;
                IntoPrimitive::into_primitive(__ret)
            }
        }
        inventory::submit! { $name::SPEC }
    };
    ($yaql_name:literal, $name:ident($p0:ident : $t0:ty, $p1:ident : $t1:ty, $p2:ident : $t2:ty) -> $ret:ty $body:block) => {
        mod $name {
            use crate::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(3), &[<$t0 as FromPrimitive>::TYPE, <$t1 as FromPrimitive>::TYPE, <$t2 as FromPrimitive>::TYPE], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
                let $p0 = match <$t0 as FromPrimitive>::from_primitive(&args[0]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let $p1 = match <$t1 as FromPrimitive>::from_primitive(&args[1]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let $p2 = match <$t2 as FromPrimitive>::from_primitive(&args[2]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let __ret: $ret = $body;
                IntoPrimitive::into_primitive(__ret)
            }
        }
        inventory::submit! { $name::SPEC }
    };
    ($yaql_name:literal, $name:ident($p0:ident : $t0:ty, $p1:ident : $t1:ty, $p2:ident : $t2:ty, $p3:ident : $t3:ty) -> $ret:ty $body:block) => {
        mod $name {
            use crate::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(4), &[<$t0 as FromPrimitive>::TYPE, <$t1 as FromPrimitive>::TYPE, <$t2 as FromPrimitive>::TYPE, <$t3 as FromPrimitive>::TYPE], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
                let $p0 = match <$t0 as FromPrimitive>::from_primitive(&args[0]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let $p1 = match <$t1 as FromPrimitive>::from_primitive(&args[1]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let $p2 = match <$t2 as FromPrimitive>::from_primitive(&args[2]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let $p3 = match <$t3 as FromPrimitive>::from_primitive(&args[3]) {
                    Some(v) => v, None => return Primitive::Null,
                };
                let __ret: $ret = $body;
                IntoPrimitive::into_primitive(__ret)
            }
        }
        inventory::submit! { $name::SPEC }
    };
}

#[macro_export]
macro_rules! yaql_raw_function {
    ($yaql_name:literal, $func:expr, $args:expr, [$($t:expr),*], $kwargs:expr) => {
        inventory::submit! {
            crate::lang::functions::Spec::new($yaql_name, $func, $args, &[$($t),*], $kwargs)
        }
    };
}

// --- Lookup ---

pub struct Functions;

impl Functions {
    pub fn lookup(&self, name: String) -> Vec<Spec> {
        inventory::iter::<Spec>()
            .filter(|s| s.name == name.as_str())
            .copied()
            .collect()
    }
}

pub static FUNCTIONS: Functions = Functions {};