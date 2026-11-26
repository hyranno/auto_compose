//! Automatic composition of short tunes.
//!
//! This crate is a port of the TypeScript implementation. Until it matches the golden fixtures
//! (`fixtures/golden/`), it reproduces the TypeScript behaviour exactly, including the order in
//! which random numbers are consumed.

pub mod error;
pub mod generator;
pub mod linalg;
pub mod markov;
pub mod random;
pub mod snapshot;
pub mod tune;
pub mod util;
