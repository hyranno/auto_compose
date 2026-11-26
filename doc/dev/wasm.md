# WebAssembly bindings

`crates/auto-composer-wasm` exposes the core crate (`crates/auto-composer`) to JavaScript.
It is a thin layer: the values crossing the boundary are plain JavaScript objects of the same shape
as the parameter JSON (`resources/*.json`) and the golden fixtures (`TuneSnapshot`).

## API

| Function | Returns |
|---|---|
| `defaultTuneParameters()` | `TuneParameters`, the contents of `resources/TuneGenParams_default.json` |
| `defaultVariantParameters()` | `VariantParameters`, the contents of `resources/VariantGenParams_default.json` |
| `generateTune(params)` | `TuneSnapshot` of the tune |
| `generateVariant(params, base)` | `TuneSnapshot` of the variant of the tune of `base` |
| `compose(params, variant)` | `TuneSnapshot` of the tune followed by its variant, as the example page plays it |

Invalid parameters (a wrong shape, or a value the generators reject) throw an `Error` with a message.
The returned objects contain no `Map`s and can be passed to `JSON.stringify` as they are.

The default parameters have a single source: the JSON files under `resources/`, which the core crate
embeds with `include_str!` as the `Default` of `generator::tune::Parameters` and `generator::variant::Parameters`.

## TypeScript declarations

The declarations in the generated `.d.ts` come from the Rust types through
[tsify](https://github.com/madonoharu/tsify). The core crate derives `Tsify` on its serialized types
when its `tsify` feature is enabled (only the wasm crate enables it).
Because several Rust types are called `Parameters` in different modules, they are renamed for
TypeScript (`TuneParameters`, `CadenceParameters`, ...) with `#[tsify(rename = ...)]`, and the fields that
refer to them name the TypeScript type with `#[tsify(type = ...)]`.
When adding a serialized type or a field of such a type, keep these attributes in line.

## Build and test

```sh
wasm-pack build crates/auto-composer-wasm --target web   # writes crates/auto-composer-wasm/pkg/
wasm-pack test --node crates/auto-composer-wasm           # golden cases through the JS boundary
```

`--target web` produces an ES module whose default export `init()` loads the `.wasm`, so a bundler
plugin is not needed.
wasm-pack takes `wasm-opt` from PATH (the devcontainer installs binaryen) and downloads
`wasm-bindgen-cli`, which must match the version in `Cargo.lock`, into its cache on first use.
Inside the Claude Code sandbox the download fails and wasm-pack builds `wasm-bindgen-cli` with `cargo install`
instead (about 30 seconds, once per cache). `.claude/settings.json` points the caches of wasm-pack and npm
to `/tmp/claude`, which the sandbox can write.

`cargo clippy --all-targets` on the host also checks the wasm crate; its tests in `tests/web.rs` only run
on `wasm32`, through `wasm-pack test`.
