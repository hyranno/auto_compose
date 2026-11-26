//! Checks that `schemas/` matches the parameter types. Regenerate with
//! `cargo run -p auto-composer --example gen_schema --features schemars`.

mod schema_support;

#[test]
fn schemas_are_up_to_date() {
    for (name, schema) in schema_support::schemas() {
        let path = schema_support::path(name);
        let file = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            file == schema,
            "{} is outdated; run `cargo run -p auto-composer --example gen_schema --features schemars`",
            path.display()
        );
    }
}
