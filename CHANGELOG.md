# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

The project was rebuilt as a Rust library with a web page on top of its WebAssembly bindings.
The same parameters no longer give the same tunes as the TypeScript version
(the last compatible commit is 83edec0).

### Added

- `auto-composer`, the Rust library crate with the generation logic.
- `auto-composer-wasm`, its WebAssembly bindings with TypeScript declarations.
- The web page under `web/` (Vite, Solid, TypeScript), with the same features as before.
  Both the tune and the variant parameters can be saved and loaded, and the resolution of the
  variant has a form.
- Golden fixtures (`fixtures/`) that pin the generated output.
- CI on GitHub Actions (formatting, clippy, tests of the crates and the bindings, and the build of the
  web page), which also deploys the page to GitHub Pages from `master`.

### Changed

- The web page is published on GitHub Pages (https://hyranno.github.io/auto_compose/) instead of
  the former site.
- The generators of the layers draw independent random numbers even when they are given the
  same seed, as in the default parameters.
- The JSON Schemas under `schemas/` are generated from the Rust types.
- The random numbers come from the standard PCG32 of `rand_pcg` instead of a reproduction of the
  npm package reliable-random, so the same seeds give different tunes.
- The format of the parameter JSON changed, and files saved before cannot be loaded:
  - The names are snake_case (`p_branch`, `abs_pitch_factor`, `is_note_on`, ...), and so are the
    cadences, the resolutions (`perfect_authentic`, ...) and the layers of a variant
    (`cadence`, `chord`, `pitch`, `rhythm`, which were numbers).
  - A tune has one seed, an integer from 0 to 2^32 - 1. Each layer (`cadence`, `chord`, `rhythm`,
    `pitch`) may have a seed of its own that replaces it; the web page has a field for each.
    The variant has one seed of the same form.
  - `note` is split into `rhythm` and `pitch` (the former `pitchWeight`), and `chord` has `roots`
    for each cadence and `tones`.
  - The weights of the cadence transitions and of the chord roots are objects keyed by cadence,
    the time measure is `{beats_per_bar, bars}`, and a timespan of a variant is `{start, end}`.
- The tunes given to the web page have a single resolution and no `max_beat_division_depth`;
  the piano roll is as fine as the shortest note.

### Fixed

- The pitch variant uses a proper transition matrix and conditions each note on the previous
  one, and keeps working when the neighbouring pitches are out of the range of candidates.
- The rhythm variant no longer fails at the ends of the tune and merges notes with the right
  neighbours.
- Playing and exporting a tune with a single chord no longer fails, and each chord lasts until
  the next one.
- An item of weight 0 in a probability table is never chosen.
- An invalid parameter file is rejected with an error instead of being used.
- A probability table whose weights are all 0, negative or not finite is an error instead of
  choosing its first item, and so is a cadence duration that is not finite.
- The Markov chains (the cadences and the pitch variant) keep their condition on the end when
  some states can never be reached, and drop it only when the end itself cannot be reached,
  instead of dropping it or choosing an arbitrary state.
- The variant parameters have their defaults before a file is loaded.
- The parameters are checked before generating, so that an invalid table, cadence duration or
  timespan is reported by the name of its field even when the seeds would not reach it.
  A variant timespan that starts before 0 is an error for every layer, including the rhythm.

### Removed

- The TypeScript implementation.
