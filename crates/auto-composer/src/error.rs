//! Errors of the generators.
//!
//! Where the TypeScript implementation throws (mostly `assertIsDefined`), the port returns an
//! [`Error`] instead of panicking, because it is called from wasm.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A weighted random choice was made from an empty list.
    NoItems,
    /// A state of a Markov chain was not found among its states.
    UnknownState,
    /// The transition probabilities of a Markov chain lack the transition `from` -> `to`
    /// (indices into the states).
    MissingTransition { from: usize, to: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoItems => write!(f, "no items to choose from"),
            Self::UnknownState => write!(f, "unknown state of the Markov chain"),
            Self::MissingTransition { from, to } => {
                write!(
                    f,
                    "missing transition probability from state {from} to state {to}"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
