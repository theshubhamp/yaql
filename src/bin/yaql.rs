use std::io::{self, Read};
use std::process::exit;
use yaql::evaluate;

const HELP: &str = "\
Usage: yaql <expr> [options]

Evaluate a YAQL expression and print the result as JSON.

Arguments:
  <expr>                 YAQL expression to evaluate. If omitted with -a,
                          expressions are read from stdin.

Options:
  -h, --help             show this help message and exit
  -d FILE, --data FILE   load JSON context from FILE
  -s, --string           treat the -d argument as a literal string context
                          (not a file path). Only meaningful with -d.
  -a, --array            read stdin line by line; each line is an expression
                          evaluated against the same context

Context is determined by, in priority order:
  1. -d FILE             (JSON file, or literal string with -s)
  2. null

Examples:
  yaql '2 + 3'
  yaql '$.a' -d data.json
  yaql '$' -d 'hello' -s
  printf '2 + 3\\n4 + 5\\n' | yaql -a
";

struct Args {
    expr: Option<String>,
    data: Option<String>,
    string: bool,
    array: bool,
}

fn parse_args() -> Args {
    let mut args = std::env::args().skip(1);
    let mut out = Args {
        expr: None,
        data: None,
        string: false,
        array: false,
    };
    let mut no_more_opts = false;
    while let Some(arg) = args.next() {
        if no_more_opts {
            if out.expr.is_none() {
                out.expr = Some(arg);
            }
            continue;
        }
        match arg.as_str() {
            "--" => no_more_opts = true,
            "-h" | "--help" => {
                print!("{}", HELP);
                exit(0);
            }
            "-s" | "--string" => out.string = true,
            "-a" | "--array" => out.array = true,
            "-d" | "--data" => {
                out.data = args.next().or_else(|| {
                    eprintln!("error: {} requires a value", arg);
                    exit(2);
                });
            }
            s if s.starts_with("--data=") => {
                out.data = Some(s["--data=".len()..].to_string());
            }
            s if s.starts_with("-d") && s.len() > 2 => {
                out.data = Some(s[2..].to_string());
            }
            s if s.starts_with('-') && s != "-" => {
                let second = s.chars().nth(1).unwrap();
                if !second.is_ascii_alphabetic() {
                    if out.expr.is_none() {
                        out.expr = Some(arg);
                    }
                    continue;
                }
                eprintln!("error: unknown option: {}", s);
                eprintln!("\n{}", HELP);
                exit(2);
            }
            _ => {
                if out.expr.is_none() {
                    out.expr = Some(arg);
                }
            }
        }
    }
    out
}

fn read_stdin() -> String {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .expect("failed to read stdin");
    buf
}

fn load_context(data: Option<String>, string: bool) -> serde_json::Value {
    match data {
        Some(d) if string => serde_json::Value::String(d),
        Some(path) => {
            let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("Unable to load data from {}", path);
                let _ = e;
                exit(1);
            });
            serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Unable to load data from {}", path);
                let _ = e;
                exit(1);
            })
        }
        None => serde_json::Value::Null,
    }
}

fn run(expr: &str, context: serde_json::Value) -> bool {
    // Returns true on success, false on error (matching Python CLI exit codes).
    let result = std::panic::catch_unwind(|| evaluate(expr, context));

    match result {
        Ok(yaql::EvalResult::Value(v)) => {
            let json = yaql::json::primitive_to_json(&v);
            println!("{}", json);
            true
        }
        Ok(yaql::EvalResult::ParseError(e)) => {
            eprintln!("Execution exception: Parse error: {}", e);
            false
        }
        Ok(yaql::EvalResult::EvalError(e)) => {
            eprintln!("Execution exception: {}", e);
            false
        }
        Err(panic) => {
            let msg = panic
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| panic.downcast_ref::<String>().map(|s| s.as_str()))
                .unwrap_or("panic (no message)");
            eprintln!("Execution exception: {}", msg);
            false
        }
    }
}

fn main() {
    let args = parse_args();
    let context = load_context(args.data, args.string);

    if args.array && args.expr.is_none() {
        // -a without positional expr: each stdin line is an expression.
        let stdin = read_stdin();
        let mut any_error = false;
        for line in stdin.lines() {
            if !run(line, context.clone()) {
                any_error = true;
            }
        }
        if any_error {
            exit(1);
        }
        return;
    }

    let expr = match args.expr {
        Some(e) => e,
        None => {
            eprintln!("error: no expression given (pass <expr> or use -a)");
            exit(2);
        }
    };

    if !run(&expr, context) {
        exit(1);
    }
}