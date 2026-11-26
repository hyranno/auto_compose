//! The golden cases under `fixtures/golden/`, shared by `tests/golden.rs` (which checks them)
//! and `examples/gen_golden.rs` (which rewrites their `expected`).

// Each user compiles this module separately and uses only part of it.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use auto_composer::generator::{tune as tune_generator, variant};
use auto_composer::random::ReliableRandomFactory;
use auto_composer::snapshot::TuneSnapshot;
use serde_json::Value;

/// The subdirectories of `fixtures/golden/`, one per kind of case.
pub const KINDS: [&str; 2] = ["tune", "variant"];

pub fn golden_dir(kind: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/golden")
        .join(kind)
}

/// The case files of a kind, sorted by file name.
pub fn case_paths(kind: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(golden_dir(kind))
        .expect("read the golden directory")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    paths
}

pub fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}

pub fn read_case(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", file_name(path)))
}

fn field<T: serde::de::DeserializeOwned>(case: &Value, key: &str) -> Result<T, String> {
    T::deserialize(&case[key]).map_err(|e| format!("{key}: {e}"))
}

/// Generates the tune of a case: `{"params"}` for `tune`, `{"params", "base"}` for `variant`.
pub fn generate(kind: &str, case: &Value) -> Result<TuneSnapshot, String> {
    let tune = match kind {
        "tune" => {
            let params: tune_generator::Parameters = field(case, "params")?;
            tune_generator::generate(&params, &ReliableRandomFactory)
        }
        "variant" => {
            let params: variant::Parameters = field(case, "params")?;
            let base: tune_generator::Parameters = field(case, "base")?;
            variant::generate(&params, &base, &ReliableRandomFactory)
        }
        _ => panic!("unknown kind {kind}"),
    }
    .map_err(|e| e.to_string())?;
    Ok(TuneSnapshot::from(&tune))
}

pub fn expected(case: &Value) -> Result<TuneSnapshot, String> {
    field(case, "expected")
}
