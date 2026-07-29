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
}