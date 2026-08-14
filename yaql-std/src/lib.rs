pub mod branching;
pub mod collections;
pub mod math;
pub mod operators;
pub mod query;
pub mod regex;
pub mod sets;
pub mod strings;

/// Register a typed stdlib function.
#[macro_export]
macro_rules! yaql_function {
    ($yaql_name:literal, $name:ident() -> $ret:ty $body:block) => {
        mod $name {
            use yaql_core::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(0), &[], false);
            pub fn func(_args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
                let __ret: $ret = $body;
                Ok(IntoPrimitive::into_primitive(__ret))
            }
        }
        inventory::submit! { $name::SPEC }
    };
    ($yaql_name:literal, $name:ident($p0:ident : $t0:ty) -> $ret:ty $body:block) => {
        mod $name {
            use yaql_core::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(1), &[<$t0 as FromPrimitive>::TYPE], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
                let $p0 = match <$t0 as FromPrimitive>::from_primitive(&args[0]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let __ret: $ret = $body;
                Ok(IntoPrimitive::into_primitive(__ret))
            }
        }
        inventory::submit! { $name::SPEC }
    };
    ($yaql_name:literal, $name:ident($p0:ident : $t0:ty, $p1:ident : $t1:ty) -> $ret:ty $body:block) => {
        mod $name {
            use yaql_core::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(2), &[<$t0 as FromPrimitive>::TYPE, <$t1 as FromPrimitive>::TYPE], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
                let $p0 = match <$t0 as FromPrimitive>::from_primitive(&args[0]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let $p1 = match <$t1 as FromPrimitive>::from_primitive(&args[1]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let __ret: $ret = $body;
                Ok(IntoPrimitive::into_primitive(__ret))
            }
        }
        inventory::submit! { $name::SPEC }
    };
    ($yaql_name:literal, $name:ident($p0:ident : $t0:ty, $p1:ident : $t1:ty, $p2:ident : $t2:ty) -> $ret:ty $body:block) => {
        mod $name {
            use yaql_core::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(3), &[<$t0 as FromPrimitive>::TYPE, <$t1 as FromPrimitive>::TYPE, <$t2 as FromPrimitive>::TYPE], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
                let $p0 = match <$t0 as FromPrimitive>::from_primitive(&args[0]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let $p1 = match <$t1 as FromPrimitive>::from_primitive(&args[1]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let $p2 = match <$t2 as FromPrimitive>::from_primitive(&args[2]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let __ret: $ret = $body;
                Ok(IntoPrimitive::into_primitive(__ret))
            }
        }
        inventory::submit! { $name::SPEC }
    };
    ($yaql_name:literal, $name:ident($p0:ident : $t0:ty, $p1:ident : $t1:ty, $p2:ident : $t2:ty, $p3:ident : $t3:ty) -> $ret:ty $body:block) => {
        mod $name {
            use yaql_core::lang::functions::*;
            pub const SPEC: Spec = Spec::new($yaql_name, func, ArgSpec::Exact(4), &[<$t0 as FromPrimitive>::TYPE, <$t1 as FromPrimitive>::TYPE, <$t2 as FromPrimitive>::TYPE, <$t3 as FromPrimitive>::TYPE], false);
            pub fn func(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Result<Primitive, EvalError> {
                let $p0 = match <$t0 as FromPrimitive>::from_primitive(&args[0]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let $p1 = match <$t1 as FromPrimitive>::from_primitive(&args[1]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let $p2 = match <$t2 as FromPrimitive>::from_primitive(&args[2]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let $p3 = match <$t3 as FromPrimitive>::from_primitive(&args[3]) {
                    Some(v) => v, None => return Ok(Primitive::Null),
                };
                let __ret: $ret = $body;
                Ok(IntoPrimitive::into_primitive(__ret))
            }
        }
        inventory::submit! { $name::SPEC }
    };
}

/// Register a raw (hand-written) stdlib function.
#[macro_export]
macro_rules! yaql_raw_function {
    ($yaql_name:literal, $func:expr, $args:expr, [$($t:expr),*], $kwargs:expr) => {
        inventory::submit! {
            yaql_core::lang::functions::Spec::new($yaql_name, $func, $args, &[$($t),*], $kwargs)
        }
    };
}
