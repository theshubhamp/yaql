pub mod branching;
pub mod collections;
pub mod math;
pub mod operators;
pub mod query;
pub mod regex;
pub mod sets;
pub mod strings;

/// Typed stdlib functions are registered with the `#[yaql_function("name")]`
/// attribute macro from the `yaql-macros` crate.

/// Register a raw (hand-written) stdlib function.
#[macro_export]
macro_rules! yaql_raw_function {
    ($yaql_name:literal, $func:expr, $args:expr, [$($t:expr),*], $kwargs:expr) => {
        inventory::submit! {
            yaql_core::lang::functions::Spec::new($yaql_name, $func, $args, &[$($t),*], $kwargs)
        }
    };
}
