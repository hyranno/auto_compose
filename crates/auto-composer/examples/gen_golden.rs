//! Rewrites the `expected` of every golden case under `fixtures/golden/` with the output of the
//! current implementation. The inputs (`params`, and `base` for variants) are kept as they are,
//! so a case is added by writing a file with its inputs and running this.
//!
//! Usage: `cargo run --example gen_golden`. Regenerate only when a change of output is intended,
//! and review the diff.

#[path = "../tests/common/mod.rs"]
mod common;

use serde_json::{Number, Value};

fn main() {
    for kind in common::KINDS {
        let paths = common::case_paths(kind);
        let mut changed = 0;
        for path in &paths {
            let name = common::file_name(path);
            let mut case = common::read_case(path);
            let actual = common::generate(kind, &case).unwrap_or_else(|e| panic!("{name}: {e}"));
            if common::expected(&case).ok() != Some(actual.clone()) {
                changed += 1;
                println!("  changed {kind}/{name}");
            }
            case["expected"] = serde_json::to_value(&actual).unwrap();
            std::fs::write(path, format(&case, "") + "\n").unwrap();
        }
        println!("{kind}: {} cases, {changed} changed", paths.len());
    }
}

fn is_flat(v: &Value) -> bool {
    match v {
        Value::Object(_) => false,
        Value::Array(a) => a.iter().all(|x| !x.is_object() && !x.is_array()),
        _ => true,
    }
}

/// A number as JavaScript's `JSON.stringify` writes it (no `.0` on integral floats).
fn number(n: &Number) -> String {
    match n.as_f64() {
        Some(f) if n.is_f64() && f.fract() == 0.0 && f.abs() < 1e21 => format!("{f:.0}"),
        _ => n.to_string(),
    }
}

/// Like `JSON.stringify(data, null, 2)`, but objects and arrays that contain no nested
/// objects or arrays (e.g. one timeline item) are kept on a single line to keep the diffs
/// readable.
fn format(data: &Value, indent: &str) -> String {
    let entries: Vec<(Option<&String>, &Value)> = match data {
        Value::Array(a) => a.iter().map(|v| (None, v)).collect(),
        Value::Object(o) => o.iter().map(|(k, v)| (Some(k), v)).collect(),
        Value::Number(n) => return number(n),
        _ => return data.to_string(),
    };
    let (open, close) = if data.is_array() {
        ("[", "]")
    } else {
        ("{", "}")
    };
    if entries.is_empty() {
        return format!("{open}{close}");
    }
    let item = |(k, v): &(Option<&String>, &Value), ind: &str| {
        let key = k.map_or(String::new(), |k| format!("{}: ", Value::from(k.as_str())));
        key + &format(v, ind)
    };
    if entries.iter().all(|(_, v)| is_flat(v)) {
        let items: Vec<String> = entries.iter().map(|e| item(e, "")).collect();
        return format!("{open}{}{close}", items.join(", "));
    }
    let inner = format!("{indent}  ");
    let items: Vec<String> = entries
        .iter()
        .map(|e| format!("{inner}{}", item(e, &inner)))
        .collect();
    format!("{open}\n{}\n{indent}{close}", items.join(",\n"))
}
