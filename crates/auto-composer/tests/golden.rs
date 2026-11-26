//! Checks the generators against `fixtures/golden/`
//! (rewrite them with `cargo run --example gen_golden`).

mod common;

use auto_composer::snapshot::TuneSnapshot;

/// Prints which timelines of a mismatching case differ and returns whether it mismatches.
fn mismatch(name: &str, actual: &TuneSnapshot, expected: &TuneSnapshot) -> bool {
    if actual == expected {
        return false;
    }
    eprintln!("{name}: mismatch");
    let actual = serde_json::to_value(actual).unwrap();
    let expected = serde_json::to_value(expected).unwrap();
    for key in ["cadence", "chord", "notes"] {
        if actual[key] != expected[key] {
            eprintln!("  {key} differs");
        }
    }
    true
}

fn check(kind: &str) {
    let paths = common::case_paths(kind);
    assert!(!paths.is_empty());
    let failed: Vec<String> = paths
        .iter()
        .map(|path| (common::file_name(path), common::read_case(path)))
        .filter(|(name, case)| {
            let actual = common::generate(kind, case).unwrap_or_else(|e| panic!("{name}: {e}"));
            let expected = common::expected(case).unwrap_or_else(|e| panic!("{name}: {e}"));
            mismatch(name, &actual, &expected)
        })
        .map(|(name, _)| name)
        .collect();
    assert!(
        failed.is_empty(),
        "{} of {} {kind} cases failed: {failed:?}",
        failed.len(),
        paths.len(),
    );
}

#[test]
fn golden_tune() {
    check("tune");
}

#[test]
fn golden_variant() {
    check("variant");
}
