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
    // ========================================================================= //
    // test_common.py
    // ========================================================================= //
    case_common_null: "null", Value::Null;
    case_common_true: "true", Value::Null;
    case_common_false: "false", Value::Null;
    case_common_string_true: "True", Value::Null;
    case_common_string_quoted: "'some string'", Value::Null;
    case_common_null_eq: "null = null", Value::Null;
    case_common_null_neq: "null != null", Value::Null;
    case_common_null_lte: "null <= null", Value::Null;
    case_common_null_gte: "null >= null", Value::Null;
    case_common_null_lt: "null < null", Value::Null;
    case_common_null_gt: "null > null", Value::Null;
    case_common_null_lt_0: "null < 0", Value::Null;
    case_common_null_lt_true: "null < true", Value::Null;
    case_common_null_lt_false: "null < false", Value::Null;
    case_common_null_lt_a: "null < a", Value::Null;
    case_common_null_lte_0: "null <= 0", Value::Null;
    case_common_null_gt_0_false: "null > 0", Value::Null;
    case_common_null_gte_0_false: "null >= 0", Value::Null;
    case_common_null_neq_0: "null != 0", Value::Null;
    case_common_null_neq_false: "null != false", Value::Null;
    case_common_null_eq_false: "null = false", Value::Null;
    case_common_null_eq_0: "null = 0", Value::Null;
    case_common_0_lt_null: "0 < null", Value::Null;
    case_common_0_lte_null: "0 <= null", Value::Null;
    case_common_0_gte_null: "0 >= null", Value::Null;
    case_common_0_gt_null: "0 > null", Value::Null;
    case_common_max_1_5: "max(1, 5)", Value::Null;
    case_common_max_null_neg1: "max(null, -1)", Value::Null;
    case_common_max_null_null: "max(null, null)", Value::Null;
    case_common_min_1_5: "min(1, 5)", Value::Null;
    case_common_min_null_neg1: "min(null, -1)", Value::Null;
    case_common_min_null_null: "min(null, null)", Value::Null;
    case_common_a_eq_1: "a = 1", Value::Null;
    case_common_a_eq_false: "a = false", Value::Null;
    case_common_a_eq_null: "a = null", Value::Null;
    case_common_list_a_eq_list_false: "[a] = [false]", Value::Null;
    case_common_a_neq_1: "a != 1", Value::Null;
    case_common_a_neq_false: "a != false", Value::Null;
    case_common_list_a_neq_list_false: "[a] != [false]", Value::Null;
    case_common_a_neq_null: "a != null", Value::Null;

    // ========================================================================= //
    // test_boolean.py
    // ========================================================================= //
    case_bool_and_tt: "true and true", Value::Null;
    case_bool_and_tf: "true and false", Value::Null;
    case_bool_and_ff: "false and false", Value::Null;
    case_bool_and_ft: "false and true", Value::Null;
    case_bool_and_12: "true and 12", Value::Null;
    case_bool_and_null_null: "null and null", Value::Null;
    case_bool_or_tt: "true or true", Value::Null;
    case_bool_or_tf: "true or false", Value::Null;
    case_bool_or_ff: "false or false", Value::Null;
    case_bool_or_ft: "false or true", Value::Null;
    case_bool_or_12: "12 or true", Value::Null;
    case_bool_or_null_null: "null or null", Value::Null;
    case_bool_not_true: "not true", Value::Null;
    case_bool_not_false: "not false", Value::Null;
    case_bool_not_0: "not 0", Value::Null;
    case_bool_not_123: "not 123", Value::Null;
    case_bool_not_empty: "not ''", Value::Null;
    case_bool_not_true_kw: "not True", Value::Null;
    case_bool_not_null: "not null", Value::Null;
    case_bool_lazy_or: "$ or 10/($-1)", json!(1);
    case_bool_lazy_and: "$ and 10/$", json!(0);
    case_bool_eq_ff: "false = false", Value::Null;
    case_bool_neq_ft: "false != true", Value::Null;
    case_bool_neq_tf: "true != false", Value::Null;
    case_bool_eq_tt: "true = true", Value::Null;
    case_bool_eq_tf_false: "true = false", Value::Null;
    case_bool_eq_ft_false: "false = true", Value::Null;
    case_bool_neq_ff_false: "false != false", Value::Null;
    case_bool_neq_tt_false: "true != true", Value::Null;
    case_bool_is_boolean_true: "isBoolean(true)", Value::Null;
    case_bool_is_boolean_false: "isBoolean(false)", Value::Null;
    case_bool_is_boolean_123: "isBoolean(123)", Value::Null;
    case_bool_is_boolean_abc: "isBoolean(abc)", Value::Null;

    // ========================================================================= //
    // test_branching.py
    // ========================================================================= //
    case_branch_switch_123: "switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)", json!(123);
    case_branch_switch_50: "switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)", json!(50);
    case_branch_switch_neg123: "switch($ < 10 => 1, $ >= 10 and $ < 100 => 2, $ >= 100 => 3)", json!(-123);
    case_branch_select_case_123: "selectCase($ < 10, $ >= 10 and $ < 100)", json!(123);
    case_branch_select_case_50: "selectCase($ < 10, $ >= 10 and $ < 100)", json!(50);
    case_branch_select_case_neg123: "selectCase($ < 10, $ >= 10 and $ < 100)", json!(-123);
    case_branch_select_all_1: "selectAllCases($ < 10, $ > 5)", json!(1);
    case_branch_select_all_7: "selectAllCases($ < 10, $ > 5)", json!(7);
    case_branch_select_all_12: "selectAllCases($ < 10, $ > 5)", json!(12);
    case_branch_examine_1: "examine($ < 10, $ > 5)", json!(1);
    case_branch_examine_7: "examine($ < 10, $ > 5)", json!(7);
    case_branch_examine_12: "examine($ < 10, $ > 5)", json!(12);
    case_branch_coalesce_null_2: "coalesce($, 2)", json!(null);
    case_branch_coalesce_1_2: "coalesce($, 2)", json!(1);
    case_branch_coalesce_null_null_2: "coalesce($, $, 2)", json!(null);
    case_branch_coalesce_null_only: "coalesce($)", json!(null);

    // ========================================================================= //
    // test_math.py
    // ========================================================================= //
    case_math_plus_int: "2 + 3", Value::Null;
    case_math_plus_float: "2 + 3.0", Value::Null;
    case_math_plus_float2: "2.3 + 3.5", Value::Null;
    case_math_minus_int: "12 - 3", Value::Null;
    case_math_minus_float: "1 - 2.1", Value::Null;
    case_math_minus_float2: "123.321 - 0.321", Value::Null;
    case_math_mul_int: "3 * 2", Value::Null;
    case_math_mul_neg: "3 * -2", Value::Null;
    case_math_mul_float: "3.0 * 2.0", Value::Null;
    case_math_div_int: "7 / 2", Value::Null;
    case_math_div_float: "5 / 2.0", Value::Null;
    case_math_div_float2: "5.0 / 2", Value::Null;
    case_math_brackets_1: "1 - (2) - 3", Value::Null;
    case_math_brackets_2: "1 - (2 - 3)", Value::Null;
    case_math_mod_int: "9 mod 5", Value::Null;
    case_math_mod_float: "9.0 mod 5", Value::Null;
    case_math_mod_float2: "9 mod 5.0", Value::Null;
    case_math_mod_float3: "9.0 mod 5.0", Value::Null;
    case_math_abs_neg: "abs(-4)", Value::Null;
    case_math_abs_pos: "abs(4)", Value::Null;
    case_math_abs_float: "abs(-4.4)", Value::Null;
    case_math_gt_5_3: "5 > 3", Value::Null;
    case_math_gt_3_3: "3 > 3", Value::Null;
    case_math_lt_3_5: "3 < 5", Value::Null;
    case_math_lt_3_3: "3 < 3", Value::Null;
    case_math_lt_float: "2.5 < 3", Value::Null;
    case_math_gte_5_3: "5 >= 3", Value::Null;
    case_math_gte_3_3: "3 >= 3", Value::Null;
    case_math_gt_float: "3.5 > 3", Value::Null;
    case_math_gte_2_3: "2 >= 3", Value::Null;
    case_math_lte_3_5: "3 <= 5", Value::Null;
    case_math_lte_3_3: "3 <= 3", Value::Null;
    case_math_lte_3_2: "3 <= 2", Value::Null;
    case_math_eq_5_5: "5 = 5", Value::Null;
    case_math_eq_1_1: "1.0 = 1", Value::Null;
    case_math_eq_5_6_false: "5 = 6", Value::Null;
    case_math_neq_5_5_false: "5 != 5", Value::Null;
    case_math_neq_0_0f_false: "0 != 0.0", Value::Null;
    case_math_neq_5_6: "5 != 6", Value::Null;
    case_math_int_str: "int('5')", Value::Null;
    case_math_int_float: "int(5.2)", Value::Null;
    case_math_int_null: "int(null)", Value::Null;
    case_math_float_str: "float('-1.23')", Value::Null;
    case_math_float_null: "float(null)", Value::Null;
    case_math_bitwise_or_1_3: "bitwiseOr(1, 3)", Value::Null;
    case_math_bitwise_or_1_2: "bitwiseOr(1, 2)", Value::Null;
    case_math_bitwise_and_1_3: "bitwiseAnd(1, 3)", Value::Null;
    case_math_bitwise_and_1_2: "bitwiseAnd(1, 2)", Value::Null;
    case_math_bitwise_xor_1_3: "bitwiseXor(1, 3)", Value::Null;
    case_math_bitwise_xor_1_2: "bitwiseXor(1, 2)", Value::Null;
    case_math_shift_left: "shiftBitsLeft(1, 5)", Value::Null;
    case_math_shift_right_32_4: "shiftBitsRight(32, 4)", Value::Null;
    case_math_shift_right_32_6: "shiftBitsRight(32, 6)", Value::Null;
    case_math_pow_2_5: "pow(2, 5)", Value::Null;
    case_math_sign_pos: "sign(123)", Value::Null;
    case_math_sign_neg: "sign(-123)", Value::Null;
    case_math_sign_zero: "sign(0)", Value::Null;
    case_math_round_23: "round(2.3)", Value::Null;
    case_math_is_integer_neg2: "isInteger(-2)", Value::Null;
    case_math_is_integer_2: "isInteger(2)", Value::Null;
    case_math_is_integer_23: "isInteger(2.3)", Value::Null;
    case_math_is_integer_abc: "isInteger(abc)", Value::Null;
    case_math_is_integer_true: "isInteger(true)", Value::Null;
    case_math_is_number_neg2: "isNumber(-2)", Value::Null;
    case_math_is_number_2: "isNumber(2)", Value::Null;
    case_math_is_number_23: "isNumber(2.3)", Value::Null;
    case_math_is_number_abc: "isNumber(abc)", Value::Null;
    case_math_is_number_true: "isNumber(true)", Value::Null;

    // ========================================================================= //
    // test_strings.py
    // ========================================================================= //
    case_str_scalar_escape: "'some \\ttext'", Value::Null;
    case_str_scalar_backslash: "'\\\\'", Value::Null;
    case_str_scalar_quote: "\"some \\\"text\\\"\"", Value::Null;
    case_str_verbatim_path: "`c:\\f\\x`", Value::Null;
    case_str_verbatim_backtick: "`\\``", Value::Null;
    case_str_verbatim_newline: "`\\n`", Value::Null;
    case_str_verbatim_backslash: "`\\\\`", Value::Null;
    case_str_len_abc: "len(abc)", Value::Null;
    case_str_to_upper_qq: "qq.toUpper()", Value::Null;
    case_str_to_lower_QQ: "QQ.toLower()", Value::Null;
    case_str_eq_a_a: "a = a", Value::Null;
    case_str_eq_a_b_false: "a = b", Value::Null;
    case_str_neq_a_a_false: "a != a", Value::Null;
    case_str_neq_a_b: "a != b", Value::Null;
    case_str_is_string_abc: "isString(abc)", Value::Null;
    case_str_is_string_null: "isString(null)", Value::Null;
    case_str_is_string_123: "isString(123)", Value::Null;
    case_str_is_string_true: "isString(true)", Value::Null;
    case_str_is_empty_str: "isEmpty('')", Value::Null;
    case_str_is_empty_null: "isEmpty(null)", Value::Null;
    case_str_is_empty_method: "null.isEmpty()", Value::Null;
    case_str_is_empty_spaces: "isEmpty('  ')", Value::Null;
    case_str_is_empty_x_false: "isEmpty('  x')", Value::Null;
    case_str_in_B_ABC: "B in ABC", Value::Null;
    case_str_in_D_ABC_false: "D in ABC", Value::Null;
    case_str_str_null: "str(null)", Value::Null;
    case_str_str_true: "str(true)", Value::Null;
    case_str_str_false: "str(false)", Value::Null;
    case_str_str_12: "str('12')", Value::Null;
    case_str_concat_plus: "a +b + c", Value::Null;
    case_str_concat_func: "concat(a, b, c)", Value::Null;
    case_str_mul_x_3: "x * 3", Value::Null;
    case_str_mul_3_x: "3 * x", Value::Null;
    case_str_max_a_z: "max(a, z)", Value::Null;
    case_str_min_a_z: "min(a, z)", Value::Null;
    case_str_to_char_array: "abc.toCharArray()", Value::Null;
    case_str_starts_with_A: "ABC.startsWith(A)", Value::Null;
    case_str_starts_with_C_false: "ABC.startsWith(C)", Value::Null;
    case_str_ends_with_C: "ABC.endsWith(C)", Value::Null;
    case_str_ends_with_B_false: "ABC.endsWith(B)", Value::Null;
    case_str_hex_255: "hex(255)", Value::Null;
    case_str_hex_neg42: "hex(-42)", Value::Null;

    // ========================================================================= //
    // test_collections.py
    // ========================================================================= //
    case_coll_list_empty: "list()", Value::Null;
    case_coll_list_123: "list(1, 2, 3)", Value::Null;
    case_coll_list_nested: "list(1, 2, list(3, 4))", Value::Null;
    case_coll_list_expr: "[1,2,3]", Value::Null;
    case_coll_list_expr_nested: "[1,[2]][1] + [3, [4]][1]", Value::Null;
    case_coll_list_expr_add: "[1,2] + [3, 4]", Value::Null;
    case_coll_list_expr_index: "([1,2] + [3, 4])[1]", Value::Null;
    case_coll_list_expr_empty: "[]", Value::Null;
    case_coll_dict_fn: "dict(a => 2, 'b c' => 13, 4 => 5, null => null, true => false, 2+6=>8)", Value::Null;
    case_coll_dict_expr: "{a => 2, 'b c' => 13, 4 => 5, null => null, true => false, 2+6=>8}", Value::Null;
    case_coll_dict_expr_add: "{a => 1} + {b=>2}", Value::Null;
    case_coll_dict_expr_empty: "{}", Value::Null;
    case_coll_index_list_0: "$[0]", json!([1, 2, 3]);
    case_coll_index_list_neg1: "$[-1]", json!([1, 2, 3]);
    case_coll_index_list_neg1_1: "$[-1-1]", json!([1, 2, 3]);
    case_coll_index_dict_a: "$[a]", json!({"a": 12, "b c": 44});
    case_coll_index_dict_bc: "$['b c']", json!({"a": 12, "b c": 44});
    case_coll_kw_dict_A: "$.A", json!({"A": 12, "b c": 44, "__d": 99, "_e": 999});
    case_coll_kw_dict_e: "$._e", json!({"A": 12, "b c": 44, "__d": 99, "_e": 999});
    case_coll_dict_get_a: "$.get(a)", json!({"a": 12, "b c": 44});
    case_coll_dict_get_b_null: "$.get(b)", json!({"a": 12, "b c": 44});
    case_coll_dict_keys: "$.keys()", json!({"a": 12, "b": 44});
    case_coll_dict_values: "$.values()", json!({"a": 12, "b": 44});
    case_coll_list_eq: "[c, 55]=[c, 55]", Value::Null;
    case_coll_list_eq_diff_false: "[c, 55]=[55, c]", Value::Null;
    case_coll_list_eq_null_false: "[c, 55]=null", Value::Null;
    case_coll_list_eq_null_rev_false: "null = [c, 55]", Value::Null;
    case_coll_list_neq_same_false: "[c, 55] != [c, 55]", Value::Null;
    case_coll_list_neq_diff: "[c, 55] != [55, c]", Value::Null;
    case_coll_list_neq_null: "[c, 55] != null", Value::Null;
    case_coll_list_neq_null_rev: "null != [c, 55]", Value::Null;
    case_coll_dict_eq: "{a => [c, 55]} = {a => [c, 55]}", Value::Null;
    case_coll_dict_eq_list_key: "{[c, 55] => a} = {[c, 55] => a}", Value::Null;
    case_coll_dict_eq_extra_false: "{[c, 55] => a, b => 1} = {[c, 55] => a}", Value::Null;
    case_coll_dict_eq_null_false: "{[c, 55] => a} = null", Value::Null;
    case_coll_dict_neq_same_false: "{a => [c, 55]} != {a => [c, 55]}", Value::Null;
    case_coll_dict_neq_list_key_same_false: "{[c, 55] => a} != {[c, 55] => a}", Value::Null;
    case_coll_dict_neq_extra: "{[c, 55] => a, b => 1} != {[c, 55] => a}", Value::Null;
    case_coll_dict_neq_null: "{[c, 55] => a} != null", Value::Null;
    case_coll_in_values: "44 in $.values()", json!({"a": 12, "b": 44});
    case_coll_in_set: "5 in set(1, 2, 5)", Value::Null;
    case_coll_in_list: "5 in [1, 2, 5]", Value::Null;
    case_coll_contains_set: "set(1, 2, 5).contains(5)", Value::Null;
    case_coll_contains_list: "[1, 2, 5].contains(5)", Value::Null;
    case_coll_list_add: "list(1, 2) + list(3, 4)", Value::Null;
    case_coll_dict_add: "dict(a => 1) + dict(b => 2)", Value::Null;
    case_coll_set_12321: "set(1, 2, 3, 2, 1)", Value::Null;
    case_coll_set_list: "set([1, 2, 3, 2, 1])", Value::Null;
    case_coll_set_empty: "set()", Value::Null;
    case_coll_set_dict: "set({a => {b => c}})", Value::Null;
    case_coll_set_len_method: "set(1, 2, 3).len()", Value::Null;
    case_coll_set_len_fn: "len(set(1, 2, 3))", Value::Null;

    // Set operations
    case_set_eq: "set(1, 2, 3) = set(3, 2, 1)", Value::Null;
    case_set_neq: "set(1, 2, 3) != set(1, 2, 3, 4)", Value::Null;
    case_to_set_list: "[1, 2, 3].toSet()", Value::Null;
    case_set_union: "set(1, 2, 3).union(set(4, 2, 3))", Value::Null;
    case_set_add_4: "set(1, 2, 3).add(4)", Value::Null;
    case_set_addition: "set(1, 2, 3) + set(4, 2, 3)", Value::Null;
    case_set_lt: "set(1, 2, 3) < set(1, 2, 3, 4)", Value::Null;
    case_set_lt_false: "set(1, 2, 3) < set(1, 2, 5)", Value::Null;
    case_set_gt: "set(1, 2, 3, 4) > set(1, 2, 3)", Value::Null;
    case_set_gt_false: "set(1, 2, 3) > set(1, 2, 3)", Value::Null;
    case_set_gte_false: "set(1, 2, 4) >= set(1, 2, 3)", Value::Null;
    case_set_gte: "set(1, 2, 3) >= set(1, 2, 3)", Value::Null;
    case_set_lte_false: "set(1, 2, 3) <= set(1, 2, 4)", Value::Null;
    case_set_lte: "set(1, 2, 3) <= set(1, 2, 3)", Value::Null;
    case_set_difference: "set(1, 2, 3, 4).difference(set(2, 3))", Value::Null;
    case_set_subtraction: "set(1, 2, 3, 4) - set(2, 3)", Value::Null;
    case_set_symmetric_diff: "set(1, 2, 3, 4).symmetricDifference(set(2, 3, 5))", Value::Null;
    case_set_add_4_5: "set(1, 2, 3).add(4, 5)", Value::Null;
    case_set_add_list: "set(1, 2, 3).add([1, 2])", Value::Null;
    case_set_add_null: "set(1, 2, 3).add(4, 5, null)", Value::Null;
    case_set_remove_2: "set(1, 2, 3).remove(2)", Value::Null;
    case_set_remove_multi: "set(1, 2, null, 3).remove(1, 2, 5)", Value::Null;
    case_set_remove_null: "set(1, 2, null, 3).remove(1, 2, 5, null)", Value::Null;
    case_set_remove_list: "set(1, 2, 3, [1, 2]).remove([1, 2])", Value::Null;
    case_set_contains: "set(1, 2, 3).contains(2)", Value::Null;
    case_is_set: "isSet(set(1, 2, 3))", Value::Null;
    case_is_set_false: "isSet([1, 2, 3])", Value::Null;
    case_is_iterable_set: "isIterable(set(1,2))", Value::Null;
    case_is_iterable_str: "isIterable(\"foo\")", Value::Null;
    case_is_iterable_dict: "isIterable({\"a\" => 1})", Value::Null;

    // ========================================================================= //
    // test_queries.py
    // ========================================================================= //
    case_q_skip: "$.skip(1)", json!([1, 2, 3, 4]);
    case_q_limit: "$.limit(2)", json!([1, 2, 3, 4]);
    case_q_take: "$.take(2)", json!([1, 2, 3, 4]);
    case_q_append: "$.append(3, 4)", json!([1, 2]);
    case_q_distinct_method: "$.distinct()", json!([1, 2, 3, 2, 4, 8]);
    case_q_distinct_fn: "distinct($)", json!([1, 2, 3, 2, 4, 8]);
    case_q_distinct_struct: "$.distinct()", json!([{"a": 1}, {"b": 2}, {"a": 1}]);
    case_q_len_fn: "len($)", json!([1, 2, 3]);
    case_q_len_method: "$.len()", json!([1, 2, 3]);
    case_q_count_method: "$.count()", json!([1, 2, 3]);
    case_q_sum_method: "$.sum()", json!([0, 1, 2, 3]);
    case_q_sum_init: "$.sum(100)", json!([0, 1, 2, 3]);
    case_q_sum_empty_init: "[].sum(100)", Value::Null;
    case_q_first: "list(2, 3).first()", Value::Null;
    case_q_last: "list(2, 3).last()", Value::Null;
    case_q_range_1: "range(2)", Value::Null;
    case_q_range_2: "range(1, 4)", Value::Null;
    case_q_max_list: "[44, 234, 23].max()", Value::Null;
    case_q_min_list: "[44, 234, 23].min()", Value::Null;
    case_q_is_iterable_empty: "isIterable([])", Value::Null;
    case_q_is_iterable_list: "isIterable([1,2])", Value::Null;
    case_q_is_iterable_num: "isIterable(1)", Value::Null;

    // ========================================================================= //
    // String methods
    // ========================================================================= //
    case_split: "$.split('\\n')", json!("some\ntext");
    case_rsplit: "$.rightSplit('\\n', 1)", json!("one\ntwo\nthree");
    case_join: "[some, text].join('-')", Value::Null;
    case_join_pythonic: "'-'.join([some, text])", Value::Null;
    case_norm_empty: "norm('')", Value::Null;
    case_norm_null: "norm(null)", Value::Null;
    case_norm_spaces: "norm('  ')", Value::Null;
    case_norm_x: "norm('  x')", Value::Null;
    case_replace: "ABBD.replace(B, x)", Value::Null;
    case_replace_count: "ABxD.replace(B, x, 1)", Value::Null;
    case_replace_dict: "AxyD.replace({x => z, y => 1})", Value::Null;
    case_replace_dict_count: "\"A122Dnull\".replace({1 => \"y\", 2 => \"false\", null => \"!\"}, 1)", Value::Null;
    case_trim: "'  x  '.trim()", Value::Null;
    case_trim_chars: "'abxba'.trim(ab)", Value::Null;
    case_trim_left: "'  x  '.trimLeft()", Value::Null;
    case_trim_left_chars: "'abxba'.trimLeft(ab)", Value::Null;
    case_trim_right: "'  x  '.trimRight()", Value::Null;
    case_trim_right_chars: "'abxba'.trimRight(ab)", Value::Null;
    case_substring_2: "$.substring(2)", json!("abcdef");
    case_substring_neg2: "$.substring(-2)", json!("abcdef");
    case_substring_2_3: "$.substring(2, 3)", json!("abcdef");
    case_substring_neg3_2: "$.substring(-3, 2)", json!("abcdef");
    case_substring_1_neg1: "$.substring(1, -1)", json!("abcdef");
    case_substring_neg5_neg1: "$.substring(-5, -1)", json!("abcdef");
    case_index_of_c: "$.indexOf(c)", json!("abcdefedcba");
    case_index_of_c_2: "$.indexOf(c, 2)", json!("abcdefedcba");
    case_index_of_x: "$.indexOf(x)", json!("abcdefedcba");
    case_index_of_f_3: "$.indexOf(f, 3)", json!("abcdefedcba");
    case_index_of_dcb_neg4_3: "$.indexOf(dcb, -4, 3)", json!("abcdefedcba");
    case_index_of_dcb_neg4_100: "$.indexOf(dcb, -4, 100)", json!("abcdefedcba");
    case_index_of_dcb_0_5: "$.indexOf(dcb, 0, 5)", json!("abcdefedcba");
    case_last_index_of_c: "$.lastIndexOf(c)", json!("abcdefedcbabc");
    case_last_index_of_c_0_4: "$.lastIndexOf(c, 0, 4)", json!("abcdefedcbabc");
    case_last_index_of_c_3_4: "$.lastIndexOf(c, 3, 4)", json!("abcdefedcbabc");
    case_last_index_of_c_neg1_1: "$.lastIndexOf(c, -1, 1)", json!("abcdefedcbabc");
    case_starts_with_2arg: "ABC.startsWith(B, A)", Value::Null;
    case_ends_with_2arg: "ABC.endsWith(B, C)", Value::Null;

    // ========================================================================= //
    // Dict/list mutation
    // ========================================================================= //
    case_dict_set: "$.set(a, 99).set(x, null)", json!({"a": 12, "b c": 44});
    case_dict_set_many: "$.set(dict(a => 55, \"d x\" => 99, null => null))", json!({"a": 12, "b c": 44});
    case_dict_set_many_inline: "$.set(a => 55, \"d x\" => 99)", json!({"a": 12, "b c": 44});
    case_dict_items: "$.items()", json!({"a": 12, "b": 44});
    case_dict_items_roundtrip: "dict($.items())", json!({"a": 12, "b": 44});
    case_dict_from_seq: "dict(list(list(a, 1), list('b', 2)))", Value::Null;
    case_index_dict_default: "$[c, 55]", json!({"a": 12, "b c": 44});
    case_index_dict_kw_default: "$[c, default => 66]", json!({"a": 12, "b c": 44});
    case_dict_get_2arg: "$.get(c, 50)", json!({"a": 12, "b c": 44});
    case_delete_dict: "$.delete(b, c)", json!({"a": 1, "b": 2, "c": 3, "d": 4});
    case_delete_all: "$.deleteAll([b, c])", json!({"a": 1, "b": 2, "c": 3, "d": 4});
    case_contains_key: "$.containsKey(a)", json!({"a": 12, "b": 44});
    case_contains_value: "$.values().contains(44)", json!({"a": 12, "b": 44});
    case_to_list_gen: "$.toList()", json!([0, 1, 2]);
    case_to_list_list: "$.toList()", json!([0, 1, 2]);

    // ========================================================================= //
    // List operations
    // ========================================================================= //
    case_list_mul_3: "3 * [1, 2]", Value::Null;
    case_list_mul_rev: "[1, 2] * 3", Value::Null;
    case_delete_0: "[1, 2, 3, 4].delete(0)", Value::Null;
    case_delete_0_2: "[1, 2, 3, 4].delete(0, 2)", Value::Null;
    case_delete_chain: "[1, 2, 3, 4].delete(0, 2).delete(0)", Value::Null;
    case_delete_1_neg1: "[1, 2, 3, 4].delete(1, -1)", Value::Null;
    case_delete_0_0: "[1, 2, 3, 4].delete(0, 0)", Value::Null;
    case_delete_0_neg1: "[1, 2, 3, 4].delete(0, -1)", Value::Null;
    case_insert_1_a: "[1, 2].insert(1, a)", Value::Null;
    case_insert_1_list: "[1, 2].insert(1, [a, b])", Value::Null;
    case_insert_neg1_a: "[1, 2].insert(-1, a)", Value::Null;
    case_insert_100_a: "[1, 2].insert(100, a)", Value::Null;
    case_insert_chain: "[].insert(0, a).insert(0, b)", Value::Null;
    case_insert_many_1: "[1, 2].insertMany(1, [a, b])", Value::Null;
    case_insert_many_neg1: "[1, 2].insertMany(-1, [a, b])", Value::Null;
    case_insert_many_100: "[1, 2].insertMany(100, [a, b])", Value::Null;
    case_insert_many_chain: "[].insertMany(0, [a, b]).insertMany(1, [a, b])", Value::Null;
    case_list_replace_0_null: "[1, 2, 3, 4].replace(0, null)", Value::Null;
    case_list_replace_0_null_2: "[1, 2, 3, 4].replace(0, null, 2)", Value::Null;
    case_list_replace_1_7_neg1: "[1, 2, 3, 4].replace(1, 7, -1)", Value::Null;
    case_replace_many_0: "[1, 2, 3, 4].replaceMany(0, [7, 8])", Value::Null;
    case_replace_many_0_2: "[1, 2, 3, 4].replaceMany(0, [7, 8], 2)", Value::Null;
    case_replace_many_1_neg1: "[1, 2, 3, 4].replaceMany(1, [7, 8], -1)", Value::Null;
    case_list_index_of: "[1, 2, 3, 2, 1].indexOf(2)", Value::Null;
    case_list_index_of_22: "[1, 2, 3, 2, 1].indexOf(22)", Value::Null;
    case_list_last_index_of: "[1, 2, 3, 2, 1].lastIndexOf(2)", Value::Null;
    case_list_last_index_of_22: "[1, 2, 3, 2, 1].lastIndexOf(22)", Value::Null;
    case_split_at: "range(1, 6).splitAt(2)", Value::Null;

    // ========================================================================= //
    // Math fixes
    // ========================================================================= //
    case_pow_3arg: "pow(2, 5, 7)", Value::Null;
    case_round_2arg: "round(2.345, 1)", Value::Null;
    case_bitwise_not: "bitwiseNot(1)", Value::Null;
    case_div_zero: "7 / 0", Value::Null;
    case_div_zero_float: "7 / -0.0", Value::Null;
    case_div_zero_0_0: "0/0", Value::Null;
    case_mod_int_neg: "9 mod -5", Value::Null;
    case_mod_float_neg: "9.1 mod -5.1", Value::Null;

    // ========================================================================= //
    // switchCase
    // ========================================================================= //
    case_switch_case_0: "$.switchCase('a', 'b', 'c')", json!(0);
    case_switch_case_1: "$.switchCase('a', 'b', 'c')", json!(1);
    case_switch_case_3: "$.switchCase('a', 'b', 'c')", json!(3);
    case_switch_case_30: "$.switchCase('a', 'b', 'c')", json!(30);
    case_switch_case_neg30: "$.switchCase('a', 'b', 'c')", json!(-30);

    // ========================================================================= //
    // test_regex.py
    // ========================================================================= //
    case_regex_matches: "regex('a.b').matches(axb)", Value::Null;
    case_regex_matches_false: "regex('a.b').matches(abx)", Value::Null;
    case_regex_method_matches: "axb.matches('a.b')", Value::Null;
    case_regex_method_matches_false: "abx.matches('a.b')", Value::Null;
    case_regex_op_match: "axb =~ regex('a.b')", Value::Null;
    case_regex_op_match_false: "abx =~ regex('a.b')", Value::Null;
    case_regex_op_not_match: "axb !~ regex('a.b')", Value::Null;
    case_regex_op_not_match_true: "abx !~ regex('a.b')", Value::Null;
    case_regex_op_str_match: "axb =~ 'a.b'", Value::Null;
    case_regex_op_str_match_false: "abx =~ 'a.b'", Value::Null;
    case_regex_op_str_not_match: "axb !~ 'a.b'", Value::Null;
    case_regex_op_str_not_match_true: "abx !~ 'a.b'", Value::Null;
    case_regex_search: "regex(`(\\d+)\\.?(\\d+)?`).search('a24.16b')", Value::Null;
    case_regex_search_all: "regex(`\\d+`).searchAll('a24.16b')", Value::Null;
    case_regex_split: "regex(`\\W+`).split('Words, words, words.')", Value::Null;
    case_regex_split_cap: "regex(`(\\W+)`).split('Words, words, words.')", Value::Null;
    case_regex_split_max: "regex(`\\W+`).split('Words, words, words.', 1)", Value::Null;
    case_regex_split_icase: "regex('[a-f]+', ignoreCase => true).split('0a3B9')", Value::Null;
    case_regex_split_on_str: "'Words, words, words.'.split(regex(`\\W+`))", Value::Null;
    case_regex_replace: "regex(`\\d+`).replace(a12b23, xx)", Value::Null;
    case_regex_replace_count: "regex(`\\d+`).replace(a12b23, xx, 1)", Value::Null;
    case_regex_replace_backref: "regex(`([a-z0-9])([A-Z])`).replace(FooBarFoo, `\\1_\\2`)", Value::Null;
    case_regex_replace_on_str: "a12b23.replace(regex(`\\d+`), xx)", Value::Null;
    case_regex_replace_on_str_count: "a12b23.replace(regex(`\\d+`), xx, 1)", Value::Null;
    case_regex_escape: "escapeRegex('[')", Value::Null;
    case_is_regex_true: "isRegex(regex(\"a.b\"))", Value::Null;
    case_is_regex_123: "isRegex(123)", Value::Null;
    case_is_regex_abc: "isRegex(abc)", Value::Null;

    // ========================================================================= //
    // Edge case fixes: first/last default, max/min init, range 3-arg,
    // enumerate, single, slice, containsKey null
    // ========================================================================= //
    case_first_null_default: "list().first(null)", Value::Null;
    case_first_default: "list().first(99)", Value::Null;
    case_last_null_default: "list().last(null)", Value::Null;
    case_last_default: "list().last(99)", Value::Null;
    case_max_empty_init: "[].max(0)", Value::Null;
    case_min_empty_init: "[].min(0)", Value::Null;
    case_range_3: "range(4, 1, -1)", Value::Null;
    case_enumerate: "$.enumerate()", json!([1, 2, 3]);
    case_enumerate_3: "$.enumerate(3)", json!([1, 2, 3]);
    case_enumerate_fn: "enumerate($)", json!([1, 2, 3]);
    case_enumerate_fn_3: "enumerate($, 3)", json!([1, 2, 3]);
    case_single_2: "list(2).single()", Value::Null;
    case_slice_range: "range(1, 6).slice(2)", Value::Null;
    case_slice_list: "[1,2,3,4,5].slice(2)", Value::Null;
    case_contains_key_null: "$.containsKey(null)", json!({"a": 12, "b": 44});

    // ========================================================================= //
    // any/all no-predicate, defaultIfEmpty
    // ========================================================================= //
    case_any_empty: "$.any()", json!([]);
    case_any_nonempty: "$.any()", json!([0]);
    case_all_empty: "$.all()", json!([]);
    case_all_nonempty: "$.all()", json!([1, 2]);
    case_all_false: "$.all()", json!([1, 0]);
    case_default_if_empty: "[].defaultIfEmpty([1, 2])", Value::Null;
    case_default_if_empty_nonempty: "[3, 4].defaultIfEmpty([1, 2])", Value::Null;

    // ========================================================================= //
    // Lambda: where, select, orderBy, takeWhile, skipWhile, any/all predicate,
    // distinct selector, indexWhere, aggregate, reduce, accumulate, toDict,
    // selectMany, concat, keyword access, leading-dash (both reject)
    // ========================================================================= //
    case_where: "$.where($ > 3)", json!([1, 2, 3, 4, 5, 6]);
    case_select: "$.select($ * $)", json!([1, 2, 3]);
    case_complex_query: "$.where($ < 4).select($ * $).skip(1).limit(1)", json!([1, 2, 3, 4, 5, 6]);
    case_order_by: "$.orderBy($)", json!([4, 2, 1, 3]);
    case_order_by_desc: "$.orderByDescending($)", json!([4, 2, 1, 3]);
    case_take_while: "[1, 2, 3, 4, 5].takeWhile($ < 4)", Value::Null;
    case_skip_while: "[1, 2, 3, 4, 5].skipWhile($ < 4)", Value::Null;
    case_all_pred: "$.all($ > 1)", json!([2, 3]);
    case_all_pred_false: "$.all($ > 1)", json!([2, 1]);
    case_distinct_sel_method: "$.distinct($[1])", json!([["a", 1], ["b", 2], ["c", 1], ["d", 3], ["e", 2]]);
    case_distinct_sel_fn: "distinct($, $[1])", json!([["a", 1], ["b", 2], ["c", 1], ["d", 3], ["e", 2]]);
    case_index_where: "[1, 2, 3, 2, 1].indexWhere($ = 2)", Value::Null;
    case_index_where_22: "[1, 2, 3, 2, 1].indexWhere($ = 22)", Value::Null;
    case_last_index_where: "[1, 2, 3, 2, 1].lastIndexWhere($ = 2)", Value::Null;
    case_last_index_where_22: "[1, 2, 3, 2, 1].lastIndexWhere($ = 22)", Value::Null;
    case_aggregate: "[a,a,b,a,a].aggregate($1 + $2)", Value::Null;
    case_reduce: "[a,a,b,a,a].reduce($1 + $2)", Value::Null;
    case_accumulate: "[a,a,b,a,a].accumulate($1 + $2)", Value::Null;
    case_to_dict: "$.toDict($, $*$)", json!([1, 2, 3]);
    case_select_many: "range(4).selectMany(range($))", Value::Null;
    case_concat_method: "$.select($).concat($.select(2 * $))", json!([1, 2, 3]);
    case_concat_fn: "concat($, $.select(2 * $), $)", json!([1, 2, 3]);
    case_keyword_access: "$.a", json!([{"a": 2}, {"a": 4}]);
    case_keyword_access_select: "$.select($).a", json!([{"a": 2}, {"a": 4}]);
    case_contains_select: "$.values().select(2*$).contains(24)", json!({"a": 12, "b": 44});
    case_in_select: "24 in $.values().select(2*$)", json!({"a": 12, "b": 44});
    case_first_select: "list(2, 3).select($ * 2).first()", Value::Null;
    case_last_select: "list(2, 3).select($ * 2).last()", Value::Null;
    case_reverse: "range(1, 4).select($*$).reverse()", Value::Null;
    case_join_seq: "[text, 1, null, true].select(str($)).join('-')", Value::Null;
    case_random: "with(random()) -> $ >= 0 and $ < 1", Value::Null;
    case_random_range: "with(random(2, 5)) -> $ >= 2 and $ <= 5", Value::Null;
    case_generate_many: "generateMany(John, $data.get($, []), decycle => true)", json!({"John": ["Jim"], "Jim": ["Jay", "Jax"], "Jax": ["John", "Jacob", "Jonathan"], "Jacob": ["Jonathan", "Jenifer"]});
    case_generate_many_dfs: "generateMany(John, $data.get($, []), decycle => true, depthFirst => true)", json!({"John": ["Jim"], "Jim": ["Jay", "Jax"], "Jax": ["John", "Jacob", "Jonathan"], "Jacob": ["Jonathan", "Jenifer"]});
    case_div_neg: "7 / -2", Value::Null;
    case_mul_float_neg: "3.1 * -2.1", Value::Null;
    case_unary_double_minus: "3--1", Value::Null;
    case_unary_float: "3.2 - -1.1", Value::Null;
    case_unary_paren: "-(1-3)", Value::Null;
    case_unary_plus_4: "+4", Value::Null;
    case_unary_plus_12f: "+12.0", Value::Null;
    case_unary_minus_plus: "3+-1", Value::Null;
    case_unary_plus_sub: "3-+1", Value::Null;
    case_unary_plus_plus: "3++1", Value::Null;
    case_unary_plus_float: "3.2 - +1.1", Value::Null;

    // ========================================================================= //
    // Lambda: aggregate/reduce/accumulate init, generate, groupBy, join,
    // zip/zipLongest, splitWhere/sliceWhere, regex selectors/replaceBy,
    // selectMany scalar
    // ========================================================================= //
    case_aggregate_init: "[].aggregate($1 + $2, 1)", Value::Null;
    case_reduce_init: "[].reduce(max($1, $2), 0)", Value::Null;
    case_accumulate_init: "[].accumulate($1 + $2, 1)", Value::Null;
    case_generate: "generate(0, $ < 10, $ + 2)", Value::Null;
    case_generate_proj: "generate(0, $ < 10, $ + 2, $ * $)", Value::Null;
    case_group_by: "$.items().orderBy($[0]).groupBy($[1])", json!({"a": 1, "b": 2, "c": 1, "d": 3, "e": 2});
    case_group_by_sel: "$.items().orderBy($[0]).groupBy($[1], $[0])", json!({"a": 1, "b": 2, "c": 1, "d": 3, "e": 2});
    case_group_by_agg: "$.items().orderBy($[0]).groupBy($[1], $[0], $.sum())", json!({"a": 1, "b": 2, "c": 1, "d": 3, "e": 2});
    case_join_self: "$.join($, $1 > $2, [$1, $2])", json!([1, 2, 3, 4]);
    case_zip_2: "[1, 2, 3].zip([4, 5])", Value::Null;
    case_zip_3: "[1, 2, 3].zip([4, 5], [6, 7, 8])", Value::Null;
    case_zip_longest_2: "[1, 2, 3].zipLongest([4, 5])", Value::Null;
    case_zip_longest_3: "[1, 2, 3].zipLongest([4, 5], [6])", Value::Null;
    case_zip_longest_default: "[1, 2, 3].zipLongest([4, 5], default => 0)", Value::Null;
    case_split_where: "range(1, 6).splitWhere($ mod 3 = 1)", Value::Null;
    case_slice_where: "[a,a,b,a,a].sliceWhere($ != a)", Value::Null;
    case_select_many_scalar: "range(2).selectMany(xx)", Value::Null;
    case_regex_search_sel: "regex(`(\\d+)\\.?(\\d+)?`).search('aa24.16bb', $.value + ' = ' + $2.value + '(' + str($2.start) + '-' + str($2.end) + ') + ' + $3.value + '(' + str($3.start) + '-' + str($3.end) + ')')", Value::Null;
    case_regex_search_all_sel: "regex(`\\d+`).searchAll('a24.16b', $.value+'!')", Value::Null;
    case_regex_replace_by: "regex(`\\d+`).replaceBy(a12b23, let(a => int($.value)) -> switch($a < 20 => xx, true => yy))", Value::Null;
    case_regex_replace_by_count: "regex(`\\d+`).replaceBy(a12b23, let(a => int($.value)) -> switch($a < 20 => xx, true => yy), 1)", Value::Null;
    case_regex_replace_by_on_str: "a12b23.replaceBy(regex(`\\d+`), with(int($.value)) -> switch($ < 20 => xx, true => yy))", Value::Null;

    // ========================================================================= //
    // Lambda: join cross, memorize, mergeWith (default/func/levels)
    // ========================================================================= //
    case_join_cross: "[1,2].join([3, 4], true, [$1, $2])", Value::Null;
    case_memorize: "let($.memorize()) -> $.len() + $.sum()", json!([0, 1, 2]);
    case_merge_with: "$.d1.mergeWith($.d2)", json!({"d1": {"a": 1, "b": "x", "c": [1, 2], "x": {"a": 1}}, "d2": {"d": 5, "b": "y", "c": [2, 3], "x": {"b": 2}}});
    case_merge_with_func: "$.d1.mergeWith($.d2, $1 + $2)", json!({"d1": {"a": 1, "b": 2, "c": [1, 2]}, "d2": {"d": 5, "b": 3, "c": [2, 3]}});
    case_merge_with_levels: "$.d1.mergeWith($.d2, $1 + $2, maxLevels => 1)", json!({"d1": {"a": 1, "b": 2, "c": [1, 2]}, "d2": {"d": 5, "b": 3, "c": [2, 3]}});

    // ========================================================================= //
    // groupBy with kwarg aggregator and empty positional arg
    // ========================================================================= //
    case_group_by_agg_kw: "$.items().orderBy($[0]).groupBy($[1], aggregator => $.sum())", json!({"a": 1, "b": 2, "c": 1, "d": 3, "e": 2});
    case_group_by_agg_no_sel: "$.items().orderBy($[0]).groupBy($[1],,  $.sum())", json!({"a": 1, "b": 2, "c": 1, "d": 3, "e": 2});

    // ========================================================================= //
    // Leading-dash expressions (both CLIs reject via optparse)
    // ========================================================================= //
    case_unary_minus_4: "-4", Value::Null;
    case_unary_minus_12f: "-12.0", Value::Null;
    case_mul_neg_neg: "-3 * -2", Value::Null;
    case_mul_float_neg_neg: "-3.1 * -2.1", Value::Null;
    case_div_float_neg: "-5.0 / 2.0", Value::Null;

    // ========================================================================= //
    // thenBy/thenByDescending (compound sort with stored sort keys)
    // ========================================================================= //
    case_order_by_thenBy: "$.orderBy($[0]).thenBy($[1])", json!([[2, 2], [1, 5], [1, 0]]);
    case_order_desc_thenBy: "$.orderByDescending($[0]).thenBy($[1])", json!([[2, 2], [1, 5], [1, 0]]);
    case_order_by_thenByDesc: "$.orderBy($[0]).thenByDescending($[1])", json!([[2, 2], [1, 5], [1, 0]]);
    case_order_desc_thenByDesc: "$.orderByDescending($[0]).thenByDescending($[1])", json!([[2, 2], [1, 5], [1, 0]]);

    // ========================================================================= //
    // repeat, cycle (infinite iterators capped at 10000)
    // ========================================================================= //
    case_repeat_2: "null.repeat(2)", Value::Null;
    case_repeat_inf: "1.repeat().limit(5)", Value::Null;
    case_cycle: "[1, 2].cycle().take(5)", Value::Null;
}

ignored_compliance_cases! {
    // Leading-dash expressions rejected by reference CLI (optparse)

    // test_math.py — random needs let/with

    // test_strings.py — split, join, norm, replace, trim, substring, indexOf, characters
    case_ignored_characters: "characters(octdigits => true, digits => true)", Value::Null;

    // test_collections.py — unimplemented features
    case_ignored_dict_list_key: "dict($ => 3).get($)", json!([1, 2]);
    case_ignored_dict_list_key_nested: "dict($ => 3).get($)", json!([1, [2]]);
    case_ignored_dict_dict_key: "dict($ => 3).get($)", json!({"a": 1});
    case_ignored_dict_eq_list_key_diff: "{[c, 55] => a} = {[c, 56] => a}", Value::Null;
    case_ignored_dict_neq_list_key_diff: "{[c, 55] => a} != {[c, 56] => a}", Value::Null;

    // test_queries.py — lambda queries not yet implemented

    // test_queries.py — other unimplemented features
    case_ignored_group_by_old: "$.items().orderBy($[0]).groupBy($[1], $[0], [$[0], $[1].sum()])", json!({"a": 1, "b": 2, "c": 1, "d": 3, "e": 2});
    case_ignored_group_by_old_no_sel: "$.items().orderBy($[0]).groupBy($[1],,  [$[0], $[1].sum()])", json!({"a": 1, "b": 2, "c": 1, "d": 3, "e": 2});
    case_ignored_group_by_old_kw: "$.items().orderBy($[0]).groupBy($[1], aggregator => [$[0], $[1].sum()])", json!({"a": 1, "b": 2, "c": 1, "d": 3, "e": 2});
    case_ignored_merge_with_min: "$.d1.mergeWith($.d2,, min($1, $2))", json!({"d1": {"a": 1, "b": 2, "c": [1, 2]}, "d2": {"d": 5, "b": 3, "c": [2, 3]}});

    // test_regex.py — regex support (lambda-dependent cases stay ignored)
}