# Golden fixtures

`fixtures/golden/` holds pairs of generator parameters and the tune they produce.
They pin down the exact output of the generators, including the random number sequence,
and are shared between the TypeScript implementation and the Rust port.
`fixtures/rng.json` pins the raw output of the random number generator itself.

## Layout

```
fixtures/rng.json              first u32 outputs of reliable-random for several seeds
fixtures/golden/
  tune/<case>.json             {"params": TuneGenerator.Parameters, "expected": TuneSnapshot}
  variant/<case>.json          {"params": VariantGenerator.Parameters, "base": TuneGenerator.Parameters, "expected": TuneSnapshot}
  variant_excluded.json        variant cases that currently throw (only present when there are any)
```

`rng.json` lists `{state, sequence, u32}` cases: `u32` are the first outputs of
`new Random(state, sequence)._random_b()`, and `random()` returns `u32 / 2^32`.
The seeds include negative values and values beyond 32 bits.
reliable-random is a PCG32 variant whose output function differs from the standard one;
`src/core/random.test.ts` contains a reference implementation of the exact algorithm.

`TuneSnapshot` is defined in `src/core/snapshot.ts`. Times and durations are sums of powers of two,
so the expected values are compared exactly.
`chord[].notenums` is redundant with `root` and `tones` and is there to cross-check the chord computation.

## Usage

- `npm test` runs `src/core/golden.test.ts`, which regenerates every case and compares it with the fixture,
  and `src/core/random.test.ts`, which checks `rng.json`.
- `npm run gen_golden` rewrites all the fixtures from the current implementation (`scripts/gen_golden.ts`,
  which also defines the cases). Regenerate only when a change of output is intended, and review the diff.
- `cargo test` runs the Rust side: `crates/auto-composer/tests/rng.rs` checks `rng.json`
  against `auto_composer::random::ReliableRandom`, `crates/auto-composer/tests/golden.rs` regenerates the
  `tune/` cases with `auto_composer::generator::tune::generate` and the `variant/` cases with
  `auto_composer::generator::variant::generate`, and compares them as `TuneSnapshot` (`snapshot.rs`),
  and the `#[cfg(test)]` modules hold the ported unit tests.
  The Rust port matches every case exactly; there is no allow-list of tolerated mismatches.
  Before committing Rust code, run `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.
- `scripts/run_ts.mjs` runs a TypeScript file through Vite's module runner,
  because `src/` uses extensionless imports that plain `node` cannot resolve.

## Tracing random number consumption

The generators take their random numbers from `Rng` (`src/core/random.ts`), created through
the optional `GenerateContext` argument of `generate` (default: reliable-random's `Random`).
Each generator asks the factory for its own `Rng` with a label
(`cadence`, `chord.root`, `chord.tone`, `rhythm`, `note.pitch`, `variant`).

When the Rust port disagrees with a fixture, record the consumption order on the TypeScript side
and compare it with the Rust side to find where they diverge:

```ts
const log: RngRecord[] = [];
TuneGenerator.generate(params, {rngFactory: recordingRngFactory(log)});
// log: [{label: 'cadence', value: 0.123...}, ...]
```

The Rust port has the same labels: each `generate` in `auto_composer::generator` takes a `&dyn RngFactory`
(`ReliableRandomFactory` normally), and `RecordingRngFactory` writes the draws to a `RefCell<Vec<RngRecord>>`.

`ScriptedRng` returns a fixed list of values, for unit tests that need specific draws (e.g. boundary values).
In Rust, the steps inside a generator (`generate_with`, `generate_roots`, `merge_top_branches`, ...) take a
`&mut impl Rng` for that purpose, and `SharedRngFactory` makes every generator draw from one `Rng`.
The variant draws from a single `Rng` (label `variant`) through all its steps
(`vary_cadence`, `vary_chord`, `interpolate`, `split_rhythm`, `merge_rhythm`, `toggle_note_on`, ...),
which also take a `&mut impl Rng`.

Seeds in the parameter JSON are read as JavaScript does: the number becomes a `f64` and then
`BigInt(x) & U64_MASK`, so integers beyond 2^53 are rounded first and a fraction is an error.

## Unit tests

Besides the fixtures, `src/core/*.test.ts` hold unit tests meant to be ported to the Rust tests.
Their inputs and expectations are written as tables for that reason.

- `util.test.ts`: `smoothstep` (including `edge0 == edge1`, which yields `NaN` exactly at the edge),
  `modulo`, `TreeNode`, `Timeline`, and `WeightedRandom` at the boundaries of the cumulative ratios.
- `tune.test.ts`, `markov.test.ts`: scales, chords, and the stationary, backward and conditional
  distributions of the default cadence chain.
- `generators.test.ts`: properties of each generator and the exact number of random draws, using
  `ScriptedRng` or a constant `Rng`.

The Rust port carries the same tables in the `#[cfg(test)]` module of the corresponding file
(`util.rs`, `tune.rs`, `markov.rs`, ...). Where TypeScript compares with `toBeCloseTo`, the Rust tests
compare exactly when the value was printed with full precision from TypeScript.
The linear algebra (`linalg.rs`) performs the same operations in the same order as the JavaScript
fallback of vectorious (LU decomposition with partial pivoting), so the Markov chain matches bit for bit.

Tests marked "known bug" record suspicious behaviour (described in a comment next to each test)
that the port reproduces on purpose until it is compatible with the fixtures. Fixing it is a separate,
intentional change that also updates the fixtures.
