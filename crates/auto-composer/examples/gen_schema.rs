//! Writes the JSON Schemas of the parameters to `schemas/`.
//!
//! ```sh
//! cargo run -p auto-composer --example gen_schema --features schemars
//! ```
//!
//! `tests/schema.rs` checks that the files are up to date.

#[path = "../tests/schema_support/mod.rs"]
mod schema_support;

fn main() {
    for (name, schema) in schema_support::schemas() {
        let path = schema_support::path(name);
        std::fs::write(&path, schema).unwrap();
        println!("wrote {}", path.display());
    }
}
