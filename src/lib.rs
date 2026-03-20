//! Blankly ID — 160-bit globally unique identifiers with embedded type hash.
//!
//! Each ID is 20 bytes: 32-bit type hash + 48-bit timestamp (ms) + 80-bit random,
//! encoded as 32 Crockford Base32 characters. IDs for different entity types are
//! inherently different thanks to the type hash segment.
//!
//! IDs can carry a human-readable type prefix separated by `-`. The prefix is
//! stripped when parsing — only the 32-char encoded value is the identity.
//!
//! # Usage
//!
//! ```
//! use bid::Bid;
//!
//! // Tagged: type hash is baked into the ID
//! let id = Bid::tagged("usr").unwrap();
//! let display = format!("usr-{id}");
//! // "usr-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
//!
//! // Parsing strips the prefix
//! let parsed: Bid = display.parse().unwrap();
//! assert_eq!(parsed, id);
//!
//! // Untagged (tag hash = 0)
//! let bare = Bid::new().unwrap();
//! ```

mod bid;
mod encoding;
pub mod error;
mod generator;

pub use bid::Bid;
pub use error::BidError;
