use thiserror::Error;

pub type Result<T> = std::result::Result<T, BidError>;

#[derive(Debug, Error)]
pub enum BidError {
    #[error("monotonic overflow: generated more than 2^80 IDs in one millisecond")]
    MonotonicOverflow,

    #[error("invalid Bid string: {0}")]
    InvalidString(String),

    #[error("invalid Bid length: expected 32 characters, got {0}")]
    InvalidLength(usize),

    #[error("random source unavailable: {0}")]
    RandomSource(String),
}
