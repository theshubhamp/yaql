use crate::lang::primitive::Primitive;
use crate::yaql_function;
use crate::yaql_raw_function;
use crate::lang::functions::ArgSpec;

// --- Typed function definitions + registration ---

yaql_function!("abs", abs_int(n: i64) -> i64 { n.abs() });
yaql_function!("abs", abs_float(n: f64) -> f64 { n.abs() });
yaql_function!("sign", sign_int(n: i64) -> i64 { n.signum() });
yaql_function!("sign", sign_float(n: f64) -> i64 { if n > 0.0 { 1 } else if n < 0.0 { -1 } else { 0 } });
yaql_function!("pow", pow_int(base: i64, exp: i64) -> i64 { base.pow(exp as u32) });

yaql_function!("pow", pow_float(base: Number, exp: Number) -> f64 {
    base.0.powf(exp.0)
});

yaql_function!("pow", pow_mod(base: i64, exp: i64, modulus: i64) -> Option<i64> {
    if modulus == 0 { None }
    else { Some(base.rem_euclid(modulus).pow(exp as u32).rem_euclid(modulus)) }
});

yaql_function!("round", round_1(n: Number) -> f64 {
    let n = n.0;
    let i = n as i64;
    let diff = n - i as f64;
    if diff > 0.5 || diff < -0.5 { n.round() }
    else if diff == 0.5 || diff == -0.5 { if i % 2 == 0 { i as f64 } else { (i + 1) as f64 } }
    else { i as f64 }
});

yaql_function!("round", round_2(n: Number, decimals: i64) -> f64 {
    let n = n.0;
    let factor = 10f64.powi(decimals as i32);
    let scaled = n * factor;
    let i = scaled as i64;
    let diff = scaled - i as f64;
    let rounded = if diff > 0.5 || diff < -0.5 { scaled.round() }
        else if diff == 0.5 || diff == -0.5 { if i % 2 == 0 { i as f64 } else { (i + 1) as f64 } }
        else { i as f64 };
    rounded / factor
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
yaql_function!("bitwiseNot", bitwise_not(a: i64) -> i64 { !a });
yaql_function!("shiftBitsLeft", shift_bits_left(a: i64, b: i64) -> i64 { a << b });
yaql_function!("shiftBitsRight", shift_bits_right(a: i64, b: i64) -> i64 { a >> b });

pub fn random_fn(args: Vec<Primitive>, _kwargs: Vec<(Primitive, Primitive)>) -> Primitive {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let mut next = || { state = state.wrapping_mul(6364136223846793005).wrapping_add(1); (state >> 33) as f64 / (1u64 << 31) as f64 };
    if args.is_empty() {
        Primitive::Float(next())
    } else if args.len() == 2 {
        let lo = match &args[0] { Primitive::Int(n) => *n, _ => return Primitive::Null };
        let hi = match &args[1] { Primitive::Int(n) => *n, _ => return Primitive::Null };
        Primitive::Int(lo + (next() * (hi - lo + 1) as f64) as i64)
    } else {
        Primitive::Null
    }
}
yaql_raw_function!("random", random_fn, ArgSpec::Min(0), [], false);