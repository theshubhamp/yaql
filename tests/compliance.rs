//! Compliance test harness: runs each YAQL expression through both the
//! reference `yaql` CLI (PyPI) and our Rust `yaql` binary, then compares
//! the outputs. Both implementations are driven through their CLI
//! (black-box, same invocation pattern) for an apples-to-apples
//! comparison. Each case is its own `#[test]` so `cargo test` reports
//! per-case pass/fail in the standard, normalised output.
//!
//! Run all:   cargo test --test compliance
//! Filter:    cargo test --test compliance -- <case name substring>
//! Show diff: cargo test --test compliance -- --nocapture <substring>

use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

    // --------------------------------------------------------------------------- //
    // CLI invocation
    // --------------------------------------------------------------------------- //

enum Outcome {
    Ok(Value),
    Err(String),
}

fn workspace_root() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().expect("cwd"))
}

fn rust_bin() -> PathBuf {
    let mut p = workspace_root();
    p.push("target");
    p.push("debug");
    p.push("yaql");
    p
}

/// Build the Command for the reference `yaql` CLI. Resolution order:
///   1. `.venv/bin/yaql` (project venv)
///   2. `yaql` on PATH (system install)
fn ref_command() -> Command {
    let venv = {
        let mut p = workspace_root();
        p.push(".venv");
        p.push("bin");
        p.push("yaql");
        p
    };
    if venv.exists() {
        return Command::new(venv);
    }
    // Check PATH for a system `yaql`.
    if let Ok(paths) = env::var("PATH") {
        for dir in paths.split(':') {
            let candidate = PathBuf::from(dir).join("yaql");
            if candidate.exists() {
                return Command::new(candidate);
            }
        }
    }
    panic!(
        "Reference yaql CLI not found. Install it via:\n  \
         python3 -m venv .venv && .venv/bin/pip install yaql\n\
         or install yaql on your system PATH."
    );
}

fn temp_context_file(context: &Value) -> Option<PathBuf> {
    if context.is_null() {
        return None;
    }
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut path = env::temp_dir();
    let pid = std::process::id();
    path.push(format!("yaql_compliance_{}_{}.json", pid, id));
    let json = serde_json::to_string(context).expect("serialize context");
    fs::write(&path, json).expect("write temp context");
    Some(path)
}

fn eval_cli(cmd: &mut Command, expr: &str, context: &Value) -> Outcome {
    let tmp = temp_context_file(context);
    if let Some(path) = &tmp {
        cmd.arg("-d").arg(path);
    }
    cmd.arg(expr);

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => return Outcome::Err(format!("failed to spawn: {}", e)),
    };
    if let Some(path) = &tmp {
        let _ = fs::remove_file(path);
    }

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        match serde_json::from_str::<Value>(&stdout) {
            Ok(v) => Outcome::Ok(v),
            Err(e) => Outcome::Err(format!("bad JSON output: {}; stdout={:?}", e, stdout)),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let msg = stderr
            .strip_prefix("Execution exception: ")
            .map(|s| s.to_string())
            .unwrap_or_else(|| if stderr.is_empty() {
                format!("exit {}", output.status.code().unwrap_or(-1))
            } else {
                stderr
            });
        Outcome::Err(msg)
    }
}

    // --------------------------------------------------------------------------- //
    // Comparison
    // --------------------------------------------------------------------------- //

fn outcomes_match(ref_out: &Outcome, rust_out: &Outcome) -> bool {
    match (ref_out, rust_out) {
        (Outcome::Err(_), Outcome::Err(_)) => true,
        (Outcome::Ok(a), Outcome::Ok(b)) => a == b,
        _ => false,
    }
}

fn format_outcome(o: &Outcome) -> String {
    match o {
        Outcome::Ok(v) => format!("Ok({})", v),
        Outcome::Err(e) => format!("Err({:?})", e),
    }
}

    // --------------------------------------------------------------------------- //
    // Per-case test generation
    // --------------------------------------------------------------------------- //

/// Assert that `expr` evaluates identically in the reference `yaql` CLI
/// and our Rust `yaql` binary, given `context`.
fn assert_case(name: &str, expr: &str, context: Value) {
    let rust = rust_bin();
    assert!(
        rust.exists(),
        "Rust binary not found at {:?} — run: cargo build --bin yaql",
        rust
    );

    let mut ref_cmd = ref_command();
    let mut rust_cmd = Command::new(&rust);
    let ref_out = eval_cli(&mut ref_cmd, expr, &context);
    let rust_out = eval_cli(&mut rust_cmd, expr, &context);
    assert!(
        outcomes_match(&ref_out, &rust_out),
        "\n  case: {}\n  expr: {:?}\n  ctx : {}\n  ref : {}\n  rust: {}",
        name,
        expr,
        context,
        format_outcome(&ref_out),
        format_outcome(&rust_out),
    );
}

/// Generates one `#[test]` per case. Each case appears as its own line in
/// `cargo test` output (e.g. `test case_add ... ok`), and failures report
/// the reference vs. rust diff inline.
macro_rules! compliance_cases {
    ( $( $name:ident : $expr:literal, $ctx:expr );* $(;)? ) => {
        $(
            #[test]
            fn $name() {
                assert_case(stringify!($name), $expr, $ctx);
            }
        )*
    };
}

/// Same as `compliance_cases!` but marks each test `#[ignore]` so that
/// known-failing cases don't break `cargo test`. Run them explicitly with
/// `cargo test --test compliance -- --ignored`. As features are implemented,
/// move cases from here into `compliance_cases!`.
macro_rules! ignored_compliance_cases {
    ( $( $name:ident : $expr:literal, $ctx:expr );* $(;)? ) => {
        $(
            #[test]
            #[ignore]
            fn $name() {
                assert_case(stringify!($name), $expr, $ctx);
            }
        )*
    };
}

compliance_cases! {
    // literals
    case_literal_int: "1", Value::Null;
    case_literal_float: "4.2", Value::Null;
    case_literal_true: "true", Value::Null;
    case_literal_false: "false", Value::Null;
    case_literal_null: "null", Value::Null;
    // data access
    case_dollar_empty: "$", json!(12);
    // comparisons
    case_eq_true: "true = true", Value::Null;
    case_eq_false: "true = false", Value::Null;
    case_neq: "true != false", Value::Null;
    case_lt: "1 < 2", Value::Null;
    case_lte: "2 <= 2", Value::Null;
    case_gt: "3 > 2", Value::Null;
    case_gte: "2 >= 2", Value::Null;
    // logical
    case_and_true: "true and true", Value::Null;
    case_and_false: "true and false", Value::Null;
    case_or_true: "false or true", Value::Null;
    case_or_false: "false or false", Value::Null;
    case_and_short_circuit_num: "true and 12", Value::Null;
    case_or_short_circuit_num: "12 or true", Value::Null;
    // precedence
    case_precedence_and_vs_eq: "1 = 1 and 2 = 2", Value::Null;
    // functions (switch with context — uses only comparison + logical ops)
    case_func_mixed_args: "switch($ < 10 => 1, $ >= 10 => 2)", json!(5);
    case_switch_high: "switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)", json!(123);
    case_switch_mid: "switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)", json!(50);
    case_switch_low: "switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)", json!(-123);

    case_math_gt: "5 > 3", Value::Null;
    case_math_lt: "3 < 5", Value::Null;
    case_math_lt_float: "2.5 < 3", Value::Null;
    case_math_gte: "5 >= 3", Value::Null;
    case_math_lte: "3 <= 5", Value::Null;
    case_math_eq_num: "5 = 5", Value::Null;
    case_math_eq_int_float: "1.0 = 1", Value::Null;
    case_math_neq_num: "5 != 6", Value::Null;
    case_math_neq_zero: "0 != 0.0", Value::Null;
    // --- upstream test_boolean.py ---
    case_bool_and_null: "null and null", Value::Null;
    case_bool_or_null: "null or null", Value::Null;
    case_common_null_neq: "null != null", Value::Null;
    case_common_null_lt: "null < null", Value::Null;
    case_common_null_gt: "null > null", Value::Null;
    // --- upstream test_branching.py ---
    case_branch_select_case: "selectCase($ < 10, $ >= 10 and $ < 100)", json!(123);
    case_branch_select_case_mid: "selectCase($ < 10, $ >= 10 and $ < 100)", json!(50);
    case_branch_select_case_low: "selectCase($ < 10, $ >= 10 and $ < 100)", json!(-123);
    case_branch_examine: "examine($ < 10, $ > 5)", json!(1);
    case_branch_examine_mid: "examine($ < 10, $ > 5)", json!(7);
    case_branch_examine_high: "examine($ < 10, $ > 5)", json!(12);
    // --- logical operators (not) ---
    case_bool_not_true: "not true", Value::Null;
    case_bool_not_false: "not false", Value::Null;
    case_bool_not_zero: "not 0", Value::Null;
    case_bool_not_num: "not 123", Value::Null;
    case_bool_not_empty_str: "not ''", Value::Null;
    case_bool_not_null: "not null", Value::Null;
    // --- boolean type checks ---
    case_bool_is_boolean_true: "isBoolean(true)", Value::Null;
    case_bool_is_boolean_false: "isBoolean(false)", Value::Null;
    case_bool_is_boolean_num: "isBoolean(123)", Value::Null;
    // --- branching (coalesce, selectAllCases) ---
    case_branch_select_all_cases: "selectAllCases($ < 10, $ > 5)", json!(1);
    case_branch_select_all_cases_mid: "selectAllCases($ < 10, $ > 5)", json!(7);
    case_branch_select_all_cases_high: "selectAllCases($ < 10, $ > 5)", json!(12);
    case_branch_coalesce_null: "coalesce($, 2)", json!(null);
    case_branch_coalesce_val: "coalesce($, 2)", json!(1);
    // --- common (null ordering, max, min, bare identifiers) ---
    case_common_null_lte: "null <= null", Value::Null;
    case_common_null_gte: "null >= null", Value::Null;
    case_common_max: "max(1, 5)", Value::Null;
    case_common_max_null: "max(null, -1)", Value::Null;
    case_common_min: "min(1, 5)", Value::Null;
    case_common_str_true: "True", Value::Null;
    case_common_str_quoted: "'some string'", Value::Null;
    // --- upstream test_math.py ---
    case_math_plus_int: "2 + 3", Value::Null;
    case_math_plus_float: "2 + 3.0", Value::Null;
    case_math_plus_float2: "2.3 + 3.5", Value::Null;
    case_math_minus_int: "12 - 3", Value::Null;
    case_math_minus_float: "1 - 2.1", Value::Null;
    case_math_mul_int: "3 * 2", Value::Null;
    case_math_mul_neg: "3 * -2", Value::Null;
    case_math_mul_float: "3.0 * 2.0", Value::Null;
    case_math_div_int: "7 / 2", Value::Null;
    case_math_div_neg: "7 / -2", Value::Null;
    case_math_div_float: "5 / 2.0", Value::Null;
    case_math_div_float2: "5.0 / 2", Value::Null;
    case_math_mod_int: "9 mod 5", Value::Null;
    case_math_mod_neg: "9 mod -5", Value::Null;
    case_math_mod_float: "9.0 mod 5", Value::Null;
    case_math_mod_float2: "9 mod 5.0", Value::Null;
    case_math_brackets: "1 - (2 - 3)", Value::Null;
    // --- upstream test_strings.py ---
    case_str_scalar_escape: "'some \\ttext'", Value::Null;
    case_str_scalar_backslash: "'\\\\'", Value::Null;
    case_str_verbatim: "`c:\\f\\x`", Value::Null;
    case_str_verbatim_backtick: "`\\``", Value::Null;
    case_str_verbatim_newline: "`\\n`", Value::Null;
    case_str_eq: "a = a", Value::Null;
    case_str_neq: "a != b", Value::Null;
    case_str_min: "min(a, z)", Value::Null;
    case_str_len: "len(abc)", Value::Null;
    case_str_to_upper: "qq.toUpper()", Value::Null;
    case_str_to_lower: "QQ.toLower()", Value::Null;
    case_str_is_string: "isString(abc)", Value::Null;
    case_str_is_string_null: "isString(null)", Value::Null;
    case_str_is_string_num: "isString(123)", Value::Null;
    case_str_concat: "a + b + c", Value::Null;
    case_str_concat_func: "concat(a, b, c)", Value::Null;
    case_str_in: "B in ABC", Value::Null;
    case_str_in_false: "D in ABC", Value::Null;
    case_str_mul: "x * 3", Value::Null;
    case_str_mul_rev: "3 * x", Value::Null;
    case_str_str_null: "str(null)", Value::Null;
    case_str_str_true: "str(true)", Value::Null;
    case_str_str_false: "str(false)", Value::Null;
    case_str_hex: "hex(255)", Value::Null;
    case_str_hex_neg: "hex(-42)", Value::Null;
    case_str_starts_with: "ABC.startsWith(A)", Value::Null;
    case_str_ends_with: "ABC.endsWith(C)", Value::Null;
    case_str_max: "max(a, z)", Value::Null;
    case_str_to_char_array: "abc.toCharArray()", Value::Null;
    case_str_is_empty: "isEmpty('')", Value::Null;
    case_str_is_empty_null: "isEmpty(null)", Value::Null;
    case_q_len: "len($)", json!([1, 2, 3]);
    // --- upstream test_collections.py ---
    case_coll_list_empty: "list()", Value::Null;
    case_coll_list: "list(1, 2, 3)", Value::Null;
    case_coll_list_nested: "list(1, 2, list(3, 4))", Value::Null;
    case_coll_dict: "dict(a => 2, 'b c' => 13, 4 => 5, null => null, true => false, 2+6 => 8)", Value::Null;
    case_coll_indexer_list: "$[0]", json!([1, 2, 3]);
    case_coll_indexer_list_neg: "$[-1]", json!([1, 2, 3]);
    case_coll_indexer_dict: "$[a]", json!({"a": 12, "b c": 44});
    case_coll_indexer_dict_str: "$['b c']", json!({"a": 12, "b c": 44});
    case_coll_dict_get: "$.get(a)", json!({"a": 12, "b c": 44});
    case_coll_dict_keys: "$.keys()", json!({"a": 12, "b": 44});
    case_coll_set: "set(1, 2, 3, 2, 1)", Value::Null;
    case_coll_dict_expr: "{a => 1}", Value::Null;
    case_coll_dict_add: "{a => 1} + {b => 2}", Value::Null;
    case_coll_dict_empty: "{}", Value::Null;
    case_coll_dict_values: "$.values()", json!({"a": 12, "b": 44});
    case_coll_list_expr: "[1,2,3]", Value::Null;
    case_coll_list_expr_empty: "[]", Value::Null;
    case_coll_list_add: "[1,2] + [3, 4]", Value::Null;
    case_coll_list_eq: "[c, 55] = [c, 55]", Value::Null;
    case_coll_list_neq: "[c, 55] != [55, c]", Value::Null;
    case_coll_dict_eq: "{a => [c, 55]} = {a => [c, 55]}", Value::Null;
    case_coll_in_list: "5 in [1, 2, 5]", Value::Null;
    case_coll_contains: "[1, 2, 5].contains(5)", Value::Null;
}





ignored_compliance_cases! {
    // --- reference CLI rejects leading-dash positional args ---
    case_math_mul_neg_neg: "-3 * -2", Value::Null;
    case_math_unary_minus: "-4", Value::Null;
    case_math_unary_minus_float: "-12.0", Value::Null;
    case_math_unary_plus: "+4", Value::Null;
    case_math_abs_neg: "abs(-4)", Value::Null;
    case_math_abs_pos: "abs(4)", Value::Null;
    case_math_abs_float: "abs(-4.4)", Value::Null;
    case_math_int_str: "int('5')", Value::Null;
    case_math_int_float: "int(5.2)", Value::Null;
    case_math_int_null: "int(null)", Value::Null;
    case_math_float_str: "float('-1.23')", Value::Null;
    case_math_float_null: "float(null)", Value::Null;
    case_math_sign_pos: "sign(123)", Value::Null;
    case_math_sign_neg: "sign(-123)", Value::Null;
    case_math_sign_zero: "sign(0)", Value::Null;
    case_math_is_integer: "isInteger(2)", Value::Null;
    case_math_is_integer_neg: "isInteger(-2)", Value::Null;
    case_math_is_integer_float: "isInteger(2.3)", Value::Null;
    case_math_is_number: "isNumber(2)", Value::Null;
    case_math_is_number_float: "isNumber(2.3)", Value::Null;
    case_math_pow: "pow(2, 5)", Value::Null;
    case_math_round: "round(2.3)", Value::Null;
    case_math_bitwise_or: "bitwiseOr(1, 3)", Value::Null;
    case_math_bitwise_and: "bitwiseAnd(1, 3)", Value::Null;
    case_math_bitwise_xor: "bitwiseXor(1, 3)", Value::Null;
    case_math_shift_left: "shiftBitsLeft(1, 5)", Value::Null;
    case_math_shift_right: "shiftBitsRight(32, 4)", Value::Null;
    case_coll_set_eq: "set(1, 2, 3) = set(3, 2, 1)", Value::Null;
    // --- upstream test_queries.py ---
    case_q_where: "$.where($ > 3)", json!([1, 2, 3, 4, 5, 6]);
    case_q_select: "$.select($ * $)", json!([1, 2, 3]);
    case_q_skip: "$.skip(1)", json!([1, 2, 3, 4]);
    case_q_take: "$.take(2)", json!([1, 2, 3, 4]);
    case_q_limit: "$.limit(2)", json!([1, 2, 3, 4]);
    case_q_distinct: "$.distinct()", json!([1, 2, 3, 2, 4, 8]);
    case_q_first: "list(2, 3).first()", Value::Null;
    case_q_last: "list(2, 3).last()", Value::Null;
    case_q_range: "range(2)", Value::Null;
    case_q_range_2: "range(1, 4)", Value::Null;
    case_q_count: "$.count()", json!([1, 2, 3]);
    case_q_sum: "$.sum()", json!([0, 1, 2, 3]);
    case_q_sum_init: "$.sum(100)", json!([0, 1, 2, 3]);
    case_q_max_list: "[44, 234, 23].max()", Value::Null;
    case_q_min_list: "[44, 234, 23].min()", Value::Null;
    case_q_reverse: "range(1, 4).select($*$).reverse()", Value::Null;
    case_q_order_by: "$.orderBy($)", json!([4, 2, 1, 3]);
    case_q_order_by_desc: "$.orderByDescending($)", json!([4, 2, 1, 3]);
    case_q_append: "$.append(3, 4)", json!([1, 2]);
    case_q_any_empty: "$.any()", json!([]);
    case_q_any_nonempty: "$.any()", json!([0]);
    case_q_all_empty: "$.all()", json!([]);
    case_q_all_nonempty: "$.all()", json!([1, 2]);
    case_q_is_iterable_list: "isIterable([])", Value::Null;
    case_q_is_iterable_num: "isIterable(1)", Value::Null;
}