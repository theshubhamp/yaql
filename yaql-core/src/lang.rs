pub mod primitive;
pub mod functions;
pub mod sets;

pub use primitive::{Primitive, truthy, as_f64, arith, compare, primitive_eq, type_rank};
pub use functions::{Functions, Function, Spec, ArgSpec, Type, FUNCTIONS, dispatch, cached_overloads};
pub use sets::{set_push_unique, is_subset, set_equal, set_difference, set_symmetric_difference};
