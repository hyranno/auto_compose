//! Errors of the generators.
//!
//! Where the TypeScript implementation throws (mostly `assertIsDefined`), the port returns an
//! [`Error`] instead of panicking, because it is called from wasm.

use std::fmt;

use crate::tune::Cadence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A weighted random choice was made from an empty list.
    NoItems,
    /// A state of a Markov chain was not found among its states.
    UnknownState,
    /// The transition probabilities of a Markov chain lack the transition `from` -> `to`
    /// (indices into the states).
    MissingTransition { from: usize, to: usize },
    /// The root probabilities of the chords lack this cadence.
    MissingRootProbabilities(Cadence),
    /// A timeline of the tune has no item at or before the time (`Timeline.get` throws in
    /// TypeScript). The name is the field of the tune, such as `"chord"`.
    NoItemAt { timeline: &'static str },
    /// A note of the tree has no pitch.
    MissingPitch,
    /// A note changed by the rhythm variant has no neighbour leaf on one side, so there is no
    /// pitch to interpolate from (TypeScript reads `.pitch` of `undefined` and throws).
    NoNeighbour,
    /// A parameter is out of its domain. The name is the field of the parameters.
    ///
    /// These are checked only where the TypeScript version would loop forever or index out of
    /// bounds; other values are accepted as they are.
    InvalidParameter(&'static str),
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
            Self::MissingRootProbabilities(c) => {
                write!(f, "no root probabilities for the cadence {}", c.as_str())
            }
            Self::NoItemAt { timeline } => write!(f, "the {timeline} timeline has no item yet"),
            Self::MissingPitch => write!(f, "a note has no pitch"),
            Self::NoNeighbour => write!(f, "no neighbour note to take the pitch from"),
            Self::InvalidParameter(name) => write!(f, "invalid parameter: {name}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
