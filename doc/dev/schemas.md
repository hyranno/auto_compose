# JSON Schemas of the parameters

`schemas/TuneGeneratorParameters.json` and `schemas/VariantGeneratorParameters.json` describe the
parameter files (`resources/*.json`) that the page loads and saves.
They are generated from the Rust types (`auto_composer::generator::{tune,variant}::Parameters`)
by [schemars](https://docs.rs/schemars), behind the `schemars` feature of the core crate.
The descriptions come from the doc comments of the types.

```sh
cargo run -p auto-composer --example gen_schema --features schemars   # rewrite schemas/
cargo test --all-features                                             # includes tests/schema.rs
```

`crates/auto-composer/tests/schema.rs` fails if a file is out of date, so run the tests with
`--all-features` after changing a parameter type, and regenerate the files if needed.
The test is skipped without the feature (`required-features` in `Cargo.toml`).

Notes:

- The generic items (`WeightedItem`, `TimelineItem`, `PerCadence`) are inlined
  (`#[schemars(inline)]`) instead of getting numbered definitions.
- The field names and the enum values are snake_case.
- A seed is an integer from 0 to 2^32 - 1, which a JavaScript number holds exactly. The seed of a
  layer (`cadence.seed`, `chord.seed`, `rhythm.seed`, `pitch.seed`) is optional and replaces the seed
  of the tune for that layer.
