//! Errors of the generators. They are returned rather than panicking, because the generators
//! are called from wasm.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A weighted random choice was made from an empty list.
    NoItems,
    /// The weights of a weighted random choice are negative or not finite, or they are all 0.
    InvalidWeights,
    /// A state of a Markov chain was not found among its states.
    UnknownState,
    /// The transition weights of a Markov chain list a state twice.
    DuplicateState,
    /// The transition probabilities of a Markov chain lack the transition `from` -> `to`
    /// (indices into the states).
    MissingTransition { from: usize, to: usize },
    /// The transition weights from a state of a Markov chain (index into the states) cannot be
    /// made into probabilities: a weight is negative or not finite, or they sum to 0.
    InvalidTransition { from: usize },
    /// A timeline of the tune has no item at or before the time. The name is the field of the
    /// tune, such as `"chord"`.
    NoItemAt { timeline: &'static str },
    /// A parameter is out of its domain. The name is the field of the parameters.
    InvalidParameter(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoItems => write!(f, "no items to choose from"),
            Self::InvalidWeights => write!(
                f,
                "the weights to choose by must be non-negative and finite, and not all 0"
            ),
            Self::UnknownState => write!(f, "unknown state of the Markov chain"),
            Self::DuplicateState => write!(f, "a state of the Markov chain is listed twice"),
            Self::MissingTransition { from, to } => {
                write!(
                    f,
                    "missing transition probability from state {from} to state {to}"
                )
            }
            Self::InvalidTransition { from } => write!(
                f,
                "the transition weights from state {from} must be non-negative and not all 0"
            ),
            Self::NoItemAt { timeline } => write!(f, "the {timeline} timeline has no item yet"),
            Self::InvalidParameter(name) => write!(f, "invalid parameter: {name}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
