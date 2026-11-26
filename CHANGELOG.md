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

### Changed

- The generators of the layers draw independent random numbers even when they are given the
  same seed, as in the default parameters.
- The JSON Schemas under `schemas/` are generated from the Rust types.

### Fixed

- The pitch variant uses a proper transition matrix and conditions each note on the previous
  one, and keeps working when the neighbouring pitches are out of the range of candidates.
- The rhythm variant no longer fails at the ends of the tune and merges notes with the right
  neighbours.
- Playing and exporting a tune with a single chord no longer fails, and each chord lasts until
  the next one.
- An invalid parameter file is rejected with an error instead of being used.
- The variant parameters have their defaults before a file is loaded.

### Removed

- The TypeScript implementation.
