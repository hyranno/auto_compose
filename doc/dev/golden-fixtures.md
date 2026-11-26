# Golden fixtures

`fixtures/golden/` holds pairs of generator parameters and the tune they produce.
They pin down the exact output of the generators, including the random number sequence,
and are shared between the TypeScript implementation and the Rust port.

## Layout

```
fixtures/golden/
  tune/<case>.json             {"params": TuneGenerator.Parameters, "expected": TuneSnapshot}
  variant/<case>.json          {"params": VariantGenerator.Parameters, "base": TuneGenerator.Parameters, "expected": TuneSnapshot}
  variant_excluded.json        variant cases that currently throw (only present when there are any)
```

`TuneSnapshot` is defined in `src/core/snapshot.ts`. Times and durations are sums of powers of two,
so the expected values are compared exactly.
`chord[].notenums` is redundant with `root` and `tones` and is there to cross-check the chord computation.

## Usage

- `npm test` runs `src/core/golden.test.ts`, which regenerates every case and compares it with the fixture.
- `npm run gen_golden` rewrites the fixtures from the current implementation (`scripts/gen_golden.ts`,
  which also defines the cases). Regenerate only when a change of output is intended, and review the diff.
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

`ScriptedRng` returns a fixed list of values, for unit tests that need specific draws (e.g. boundary values).
