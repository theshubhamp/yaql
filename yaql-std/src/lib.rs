//! Typed stdlib functions are registered with the `#[yaql_function("name")]`
//! attribute macro from the `yaql-macros` crate. Raw (hand-written) stdlib
//! functions are registered with the `#[yaql_function(...)]` attribute
//! macro from the same crate.

pub mod branching;
pub mod collections;
pub mod math;
pub mod operators;
pub mod query;
pub mod regex;
pub mod sets;
pub mod strings;
