//! The JSON Schemas under `schemas/`, shared by `tests/schema.rs` and `examples/gen_schema.rs`.

use std::path::PathBuf;

use auto_composer::generator::{tune, variant};

/// The file names (without `.json`) and contents of the schemas.
pub fn schemas() -> Vec<(&'static str, String)> {
    vec![
        (
            "TuneGeneratorParameters",
            to_string(schemars::schema_for!(tune::Parameters)),
        ),
        (
            "VariantGeneratorParameters",
            to_string(schemars::schema_for!(variant::Parameters)),
        ),
    ]
}

pub fn path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas")
        .join(format!("{name}.json"))
}

fn to_string(schema: schemars::Schema) -> String {
    serde_json::to_string_pretty(&schema).unwrap() + "\n"
}
