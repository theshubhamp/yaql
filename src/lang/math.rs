use crate::lang::primitive::{Primitive, as_f64};
use crate::lang::functions::{FromPrimitive, IntoPrimitive, Number, Any, Null, ArgSpec, Type, Spec};
use crate::yaql_function;
use crate::yaql_raw_function;

// --- Typed function definitions + registration ---

yaql_function!("abs", abs_int(n: i64) -> i64 { n.abs() });
yaql_function!("abs", abs_float(n: f64) -> f64 { n.abs() });
yaql_function!("sign", sign_int(n: i64) -> i64 { n.signum() });
yaql_function!("sign", sign_float(n: f64) -> i64 { if n > 0.0 { 1 } else if n < 0.0 { -1 } else { 0 } });
yaql_function!("pow", pow_int(base: i64, exp: i64) -> i64 { base.pow(exp as u32) });

// pow_float needs as_f64 which works on both Int and Float
pub fn pow_float(args: Vec<Primitive>, kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    let b = as_f64(&args[0]);
    let e = as_f64(&args[1]);
    match (b, e) {
        (Some(b), Some(e)) => Primitive::Float(b.powf(e)),
        _ => Primitive::Null,
    }
}
yaql_raw_function!("pow", pow_float, ArgSpec::Exact(2), [Type::Number, Type::Number], false);

yaql_function!("round", round(n: Number) -> f64 {
    let n = n.0;
    let i = n as i64;
    let diff = n - i as f64;
    if diff > 0.5 || diff < -0.5 { n.round() }
    else if diff == 0.5 || diff == -0.5 { if i % 2 == 0 { i as f64 } else { (i + 1) as f64 } }
    else { i as f64 }
});

yaql_function!("int", int_from_int(n: i64) -> i64 { n });
yaql_function!("int", int_from_float(n: f64) -> i64 { n as i64 });
yaql_function!("int", int_from_string(s: String) -> i64 {
    s.trim().parse::<i64>().unwrap_or_else(|_| {
        s.trim().parse::<f64>().map(|f| f as i64).unwrap_or(0)
    })
});
yaql_function!("int", int_from_null(_n: Null) -> i64 { 0 });

yaql_function!("float", float_from_int(n: i64) -> f64 { n as f64 });
yaql_function!("float", float_from_float(n: f64) -> f64 { n });
yaql_function!("float", float_from_string(s: String) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0)
});
yaql_function!("float", float_from_null(_n: Null) -> f64 { 0.0 });

yaql_function!("isInteger", is_integer(v: Any) -> bool { matches!(v.0, Primitive::Int(_)) });
yaql_function!("isNumber", is_number(v: Any) -> bool { matches!(v.0, Primitive::Int(_) | Primitive::Float(_)) });

yaql_function!("bitwiseOr", bitwise_or(a: i64, b: i64) -> i64 { a | b });
yaql_function!("bitwiseAnd", bitwise_and(a: i64, b: i64) -> i64 { a & b });
yaql_function!("bitwiseXor", bitwise_xor(a: i64, b: i64) -> i64 { a ^ b });
yaql_function!("shiftBitsLeft", shift_bits_left(a: i64, b: i64) -> i64 { a << b });
yaql_function!("shiftBitsRight", shift_bits_right(a: i64, b: i64) -> i64 { a >> b });