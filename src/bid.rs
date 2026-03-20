//! The core Bid type.

use crate::encoding;
use crate::error::BidError;
use std::fmt;
use std::str::FromStr;

/// A Blankly ID: 160-bit globally unique identifier with embedded type hash.
///
/// Layout (big-endian, 20 bytes):
/// - Bytes 0..4 (32 bits): FNV-1a hash of the type tag
/// - Bytes 4..10 (48 bits): Unix timestamp in milliseconds
/// - Bytes 10..20 (80 bits): Random/monotonic component
///
/// Encoded as 32 Crockford Base32 characters. The type tag hash means IDs
/// created for different entity types are inherently different.
///
/// String format: `tag-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX` where `tag` is an
/// optional human-readable prefix (stripped on parse) and the 32 chars encode
/// the full 160-bit value.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bid([u8; 20]);

impl Bid {
    /// Generate a new ID tagged with an entity type (e.g., "usr", "post").
    ///
    /// The tag is hashed into the ID — different tags produce different IDs.
    pub fn tagged(tag: &str) -> crate::error::Result<Bid> {
        let hash = fnv1a_32(tag.as_bytes());
        let mut inner = crate::generator::generate()?;
        inner.0[0..4].copy_from_slice(&hash.to_be_bytes());
        Ok(inner)
    }

    /// Generate a new ID with no type tag (tag hash = 0).
    pub fn new() -> crate::error::Result<Bid> {
        crate::generator::generate()
    }

    /// Construct from parts.
    pub fn from_parts(tag_hash: u32, timestamp_ms: u64, random: &[u8; 10]) -> Self {
        let mut bytes = [0u8; 20];

        bytes[0..4].copy_from_slice(&tag_hash.to_be_bytes());

        bytes[4] = (timestamp_ms >> 40) as u8;
        bytes[5] = (timestamp_ms >> 32) as u8;
        bytes[6] = (timestamp_ms >> 24) as u8;
        bytes[7] = (timestamp_ms >> 16) as u8;
        bytes[8] = (timestamp_ms >> 8) as u8;
        bytes[9] = timestamp_ms as u8;

        bytes[10..20].copy_from_slice(random);

        Bid(bytes)
    }

    /// The nil (all-zero) Bid.
    pub const fn nil() -> Self {
        Bid([0u8; 20])
    }

    /// Whether this Bid is the nil value.
    pub fn is_nil(&self) -> bool {
        self.0 == [0u8; 20]
    }

    /// The 32-bit tag hash.
    pub fn tag_hash(&self) -> u32 {
        u32::from_be_bytes([self.0[0], self.0[1], self.0[2], self.0[3]])
    }

    /// Extract the 48-bit Unix timestamp in milliseconds.
    pub fn timestamp_ms(&self) -> u64 {
        ((self.0[4] as u64) << 40)
            | ((self.0[5] as u64) << 32)
            | ((self.0[6] as u64) << 24)
            | ((self.0[7] as u64) << 16)
            | ((self.0[8] as u64) << 8)
            | (self.0[9] as u64)
    }

    /// Extract the 80-bit random component.
    pub fn random(&self) -> [u8; 10] {
        let mut r = [0u8; 10];
        r.copy_from_slice(&self.0[10..20]);
        r
    }

    /// Borrow the raw 20 bytes.
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Consume into raw 20 bytes.
    pub fn to_bytes(self) -> [u8; 20] {
        self.0
    }

    /// Compute the tag hash for a given tag string (FNV-1a).
    pub fn hash_tag(tag: &str) -> u32 {
        fnv1a_32(tag.as_bytes())
    }
}

/// FNV-1a 32-bit hash. Simple, fast, no dependencies.
fn fnv1a_32(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

// ── Display / Parse ──────────────────────────────────────────────────

impl fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = encoding::encode(&self.0);
        let s = std::str::from_utf8(&encoded).unwrap();
        f.write_str(s)
    }
}

impl fmt::Debug for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bid({})", self)
    }
}

impl FromStr for Bid {
    type Err = BidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Strip optional type prefix: everything before the first '-'
        let base32 = match s.find('-') {
            Some(pos) => &s[pos + 1..],
            None => s,
        };

        if base32.len() != 32 {
            return Err(BidError::InvalidLength(base32.len()));
        }

        let input: [u8; 32] = base32.as_bytes().try_into().unwrap();
        let bytes = encoding::decode(&input)?;
        Ok(Bid(bytes))
    }
}

// ── Conversions ──────────────────────────────────────────────────────

impl From<[u8; 20]> for Bid {
    fn from(bytes: [u8; 20]) -> Self {
        Bid(bytes)
    }
}

impl From<Bid> for [u8; 20] {
    fn from(bid: Bid) -> [u8; 20] {
        bid.0
    }
}

impl AsRef<[u8]> for Bid {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// ── Serde ────────────────────────────────────────────────────────────

#[cfg(feature = "serde")]
impl serde::Serialize for Bid {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Bid {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            let s = <String as serde::Deserialize>::deserialize(deserializer)?;
            s.parse().map_err(serde::de::Error::custom)
        } else {
            struct BidVisitor;

            impl<'de> serde::de::Visitor<'de> for BidVisitor {
                type Value = Bid;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("20 bytes")
                }

                fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Bid, E> {
                    let bytes: [u8; 20] =
                        v.try_into().map_err(|_| E::invalid_length(v.len(), &self))?;
                    Ok(Bid(bytes))
                }

                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    mut seq: A,
                ) -> Result<Bid, A::Error> {
                    let mut bytes = [0u8; 20];
                    for (i, byte) in bytes.iter_mut().enumerate() {
                        *byte = seq
                            .next_element()?
                            .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                    }
                    Ok(Bid(bytes))
                }
            }

            deserializer.deserialize_bytes(BidVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagged() {
        let bid = Bid::tagged("usr").unwrap();
        assert!(!bid.is_nil());
        assert_ne!(bid.tag_hash(), 0);
        assert_eq!(bid.to_string().len(), 32);
    }

    #[test]
    fn test_untagged() {
        let bid = Bid::new().unwrap();
        assert_eq!(bid.tag_hash(), 0);
        assert_eq!(bid.to_string().len(), 32);
    }

    #[test]
    fn test_different_tags_produce_different_ids() {
        let a = Bid::tagged("usr").unwrap();
        let b = Bid::tagged("post").unwrap();
        // Tag hashes are definitely different
        assert_ne!(a.tag_hash(), b.tag_hash());
        // And therefore the Bids are different
        assert_ne!(a, b);
    }

    #[test]
    fn test_tag_hash_is_deterministic() {
        assert_eq!(Bid::hash_tag("usr"), Bid::hash_tag("usr"));
        assert_ne!(Bid::hash_tag("usr"), Bid::hash_tag("post"));
    }

    #[test]
    fn test_from_parts_and_accessors() {
        let hash = fnv1a_32(b"usr");
        let ts: u64 = 1_700_000_000_000;
        let random = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let bid = Bid::from_parts(hash, ts, &random);

        assert_eq!(bid.tag_hash(), hash);
        assert_eq!(bid.timestamp_ms(), ts);
        assert_eq!(bid.random(), random);
    }

    #[test]
    fn test_nil() {
        let bid = Bid::nil();
        assert!(bid.is_nil());
        assert_eq!(bid.tag_hash(), 0);
        assert_eq!(bid.timestamp_ms(), 0);
        assert_eq!(bid.to_string(), "00000000000000000000000000000000");
    }

    #[test]
    fn test_tagged_display_format() {
        let bid = Bid::tagged("usr").unwrap();
        let tagged = format!("usr-{bid}");
        assert!(tagged.starts_with("usr-"));
        assert_eq!(tagged.len(), 36); // "usr-" (4) + 32 base32
    }

    #[test]
    fn test_parse_bare() {
        let bid = Bid::tagged("usr").unwrap();
        let s = bid.to_string();
        let parsed: Bid = s.parse().unwrap();
        assert_eq!(parsed, bid);
    }

    #[test]
    fn test_parse_with_prefix() {
        let bid = Bid::tagged("usr").unwrap();
        let tagged = format!("usr-{bid}");
        let parsed: Bid = tagged.parse().unwrap();
        assert_eq!(parsed, bid);
    }

    #[test]
    fn test_parse_any_prefix() {
        let bid = Bid::tagged("usr").unwrap();
        let s = bid.to_string();
        let parsed: Bid = format!("anything-{s}").parse().unwrap();
        assert_eq!(parsed, bid);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let bid = Bid::tagged("usr").unwrap();
        let lower = bid.to_string().to_lowercase();
        let parsed: Bid = lower.parse().unwrap();
        assert_eq!(parsed, bid);
    }

    #[test]
    fn test_parse_invalid() {
        assert!("tooshort".parse::<Bid>().is_err());
        assert!("".parse::<Bid>().is_err());
        assert!("usr-tooshort".parse::<Bid>().is_err());
    }

    #[test]
    fn test_ord_groups_by_tag_hash() {
        // Different tag hashes sort by hash value
        let hash_a = Bid::hash_tag("aaa");
        let hash_b = Bid::hash_tag("bbb");

        let a = Bid::from_parts(hash_a, 2000, &[0; 10]);
        let b = Bid::from_parts(hash_b, 1000, &[0; 10]);

        // Sort by tag hash first, regardless of timestamp
        if hash_a < hash_b {
            assert!(a < b);
        } else {
            assert!(a > b);
        }
    }

    #[test]
    fn test_ord_chronological_within_same_tag() {
        let hash = Bid::hash_tag("usr");
        let earlier = Bid::from_parts(hash, 1000, &[0; 10]);
        let later = Bid::from_parts(hash, 2000, &[0; 10]);
        assert!(earlier < later);
    }

    #[test]
    fn test_bytes_roundtrip() {
        let bid = Bid::tagged("usr").unwrap();
        let bytes: [u8; 20] = bid.into();
        let back = Bid::from(bytes);
        assert_eq!(bid, back);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_json_roundtrip() {
        let bid = Bid::tagged("usr").unwrap();
        let json = serde_json::to_string(&bid).unwrap();

        assert!(json.starts_with('"'));
        assert_eq!(json.len(), 34); // 32 + 2 quotes

        let parsed: Bid = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, bid);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_json_deserialize_tagged() {
        let bid = Bid::tagged("usr").unwrap();
        let tagged_json = format!("\"usr-{}\"", bid);
        let parsed: Bid = serde_json::from_str(&tagged_json).unwrap();
        assert_eq!(parsed, bid);
    }
}
