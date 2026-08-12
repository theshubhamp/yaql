//! Performance test: measures evaluation throughput across a range of
//! dispatch scenarios, from basic literals up to complex chained expressions.
//!
//! This is intentionally *not* an exhaustive comparative harness (see
//! tests/compliance.rs for correctness). Instead it picks a representative
//! slice of dispatch patterns — basic cases first, then gradually more
//! complex ones — and reports ops/sec so regressions in the hot paths
//! (operator dispatch, typed overload selection, lambda evaluation, regex,
//! method chains) are visible.
//!
//! Context is fed as a Rust-native `Primitive` (via `evaluate_with`) so the
//! measurement reflects dispatch/evaluation cost rather than JSON parsing.
//!
//! Timings are machine- and build-profile-dependent, so this test makes no
//! pass/fail assertion on throughput. Run it explicitly (it is #[ignore]):
//!
//!   cargo test --release --test perf -- --ignored --nocapture
//!
//! Release mode matters: debug builds are dominated by unoptimized dispatch.

use std::collections::HashMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use yaql::lang::Primitive;

fn eval_ok(expr: &str, ctx: &Primitive) -> bool {
    matches!(yaql::evaluate_with(expr, ctx.clone()), yaql::EvalResult::Value(_))
}

/// Measure ops/sec for `expr` evaluated against `ctx`, running until at least
/// `min_time_ms` elapses. Warmup + validation first.
fn bench(name: &str, expr: &str, ctx: &Primitive, min_time_ms: u64) {
    assert!(
        eval_ok(expr, ctx),
        "scenario '{name}' did not evaluate to a value: {expr:?}"
    );

    for _ in 0..10_000 {
        black_box(eval_ok(expr, ctx));
    }

    let target = Duration::from_millis(min_time_ms);
    let start = Instant::now();
    let mut iters: u64 = 0;
    let mut acc: u64 = 0;
    while start.elapsed() < target {
        for _ in 0..10_000 {
            if black_box(eval_ok(expr, ctx)) {
                acc += 1;
            }
            iters += 1;
        }
    }
    let elapsed = start.elapsed();
    let ops = iters as f64 / elapsed.as_secs_f64();
    black_box(acc);
    println!("  {:<44} {:>14.0} ops/s", name, ops);
}

fn context() -> Primitive {
    let mut data = HashMap::new();
    data.insert("x".to_string(), Primitive::Int(5));
    let mut nested = HashMap::new();
    nested.insert("y".to_string(), Primitive::Array(vec![Primitive::Int(1), Primitive::Int(2), Primitive::Int(3)]));
    data.insert("nested".to_string(), Primitive::Map(nested));

    let users = Primitive::Array(vec![
        Primitive::Map(map(&[("name", Primitive::String("alice".to_string())), ("age", Primitive::Int(30))])),
        Primitive::Map(map(&[("name", Primitive::String("bob".to_string())), ("age", Primitive::Int(25))])),
        Primitive::Map(map(&[("name", Primitive::String("carol".to_string())), ("age", Primitive::Int(35))])),
    ]);
    let root = HashMap::from([
        ("a".to_string(), Primitive::Int(1)),
        ("name".to_string(), Primitive::String("world".to_string())),
        ("greeting".to_string(), Primitive::String("Hello, World!".to_string())),
        ("flag".to_string(), Primitive::Boolean(true)),
        ("arr".to_string(), Primitive::Array((1..=10).map(Primitive::Int).collect())),
        ("big".to_string(), Primitive::Array((1..=10).map(|i| Primitive::Int(i * 10)).collect())),
        ("strs".to_string(), Primitive::Array(vec!["apple", "banana", "cherry", "date", "elderberry"].into_iter().map(|s| Primitive::String(s.to_string())).collect())),
        ("s".to_string(), Primitive::String("the quick brown fox jumps over the lazy dog".to_string())),
        ("data".to_string(), Primitive::Map(data)),
        ("users".to_string(), users),
    ]);
    Primitive::Map(root)
}

fn map(pairs: &[(&str, Primitive)]) -> HashMap<String, Primitive> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}
#[test]
#[ignore]
fn dispatch_throughput() {
    let ctx = context();

    println!("--- dispatch throughput (ops/s, higher is better) ---");
    let min = 250;

    // Basic: literals and single operators (no dispatch / minimal dispatch).
    bench("literal int", "1", &Primitive::Null, min);
    bench("literal string", "'hello'", &Primitive::Null, min);
    bench("binary add (int)", "1 + 2", &Primitive::Null, min);
    bench("binary compare", "3 > 2", &Primitive::Null, min);
    bench("boolean and (short-circuit)", "true and false", &Primitive::Null, min);

    // Basic lookup / collection access.
    bench("dollar lookup", "$.a", &ctx, min);
    bench("dot access", "$.data.x", &ctx, min);
    bench("index + dot", "$.users[0].name", &ctx, min);
    bench("dict index", "$.data['x']", &ctx, min);

    // Simple typed stdlib functions.
    bench("len(array)", "len($.arr)", &ctx, min);
    bench("str(int)", "str(42)", &Primitive::Null, min);
    bench("toUpper", "toUpper($.s)", &ctx, min);
    bench("string substring", "$.s.substring(4, 5)", &ctx, min);

    // Overloaded typed functions (exercises specificity sort in dispatch).
    bench("max varargs", "max(1, 5, 3, 9, 2)", &Primitive::Null, min);
    bench("indexOf string", "indexOf('hello world', 'o')", &Primitive::Null, min);
    bench("contains array", "$.arr.contains(7)", &ctx, min);

    // Varargs functions.
    bench("list varargs", "list(1, 2, 3, 4, 5)", &Primitive::Null, min);
    bench("concat strings", "concat('a', 'b', 'c', 'd')", &Primitive::Null, min);
    bench("switch varargs", "switch(1 = 2 => 'no', 2 = 2 => 'yes')", &Primitive::Null, min);

    // Lambda dispatch.
    bench("select identity lambda", "$.arr.select($)", &ctx, min);
    bench("where predicate lambda", "$.arr.where($ > 5)", &ctx, min);
    bench("select lambda arithmetic", "$.arr.select($ * 2)", &ctx, min);
    bench("aggregate 2-arg lambda", "$.arr.aggregate($1 + $2)", &ctx, min);
    bench("sum overload", "$.arr.sum()", &ctx, min);

    // Regex.
    bench("regex match operator", "$.s =~ '^the '", &ctx, min);
    bench("regex search", "regex('^[a-z]+$').search('hello')", &Primitive::Null, min);

    // Method chains / progressively complex expressions.
    bench("method chain (map+filter+sum)", "$.arr.select($ * 2).where($ > 5).sum()", &ctx, min);
    bench("sort + take", "$.arr.orderBy($).take(5).sum()", &ctx, min);
    bench("users map ages", "$.users.select($.age).aggregate($1 + $2)", &ctx, min);
    bench("nested dict + list", "$.data.nested.y.select($ + 1).sum()", &ctx, min);
    bench("lambda w/ field access", "$.users.where($.age > 26).select($.name)", &ctx, min);
    bench("deep composite", "$.arr.select($ * 2).where($ > 5).orderByDescending($).take(3).sum()", &ctx, min);

    println!("--- done ---");
}
