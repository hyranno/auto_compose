//! Automatic composition of short tunes.
//!
//! This crate is a port of the TypeScript implementation. It reproduces the TypeScript behaviour
//! exactly, including the order in which random numbers are consumed and the known bugs, and
//! matches every case of the golden fixtures (`fixtures/golden/`). Behaviour changes are made
//! on purpose afterwards, together with an update of the fixtures.

pub mod error;
pub mod generator;
pub mod linalg;
pub mod markov;
pub mod random;
pub mod snapshot;
pub mod tune;
pub mod util;
