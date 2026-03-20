//! Crockford Base32 encoding/decoding for 160-bit values (20 bytes → 32 chars).

use crate::error::BidError;

/// Crockford's Base32 alphabet (excludes I, L, O, U).
const ENCODE_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Decode table: maps ASCII byte -> 5-bit value, or 0xFF for invalid.
const DECODE_TABLE: [u8; 128] = {
    let mut table = [0xFFu8; 128];

    table[b'0' as usize] = 0;
    table[b'1' as usize] = 1;
    table[b'2' as usize] = 2;
    table[b'3' as usize] = 3;
    table[b'4' as usize] = 4;
    table[b'5' as usize] = 5;
    table[b'6' as usize] = 6;
    table[b'7' as usize] = 7;
    table[b'8' as usize] = 8;
    table[b'9' as usize] = 9;

    table[b'A' as usize] = 10;
    table[b'B' as usize] = 11;
    table[b'C' as usize] = 12;
    table[b'D' as usize] = 13;
    table[b'E' as usize] = 14;
    table[b'F' as usize] = 15;
    table[b'G' as usize] = 16;
    table[b'H' as usize] = 17;
    table[b'J' as usize] = 18;
    table[b'K' as usize] = 19;
    table[b'M' as usize] = 20;
    table[b'N' as usize] = 21;
    table[b'P' as usize] = 22;
    table[b'Q' as usize] = 23;
    table[b'R' as usize] = 24;
    table[b'S' as usize] = 25;
    table[b'T' as usize] = 26;
    table[b'V' as usize] = 27;
    table[b'W' as usize] = 28;
    table[b'X' as usize] = 29;
    table[b'Y' as usize] = 30;
    table[b'Z' as usize] = 31;

    table[b'a' as usize] = 10;
    table[b'b' as usize] = 11;
    table[b'c' as usize] = 12;
    table[b'd' as usize] = 13;
    table[b'e' as usize] = 14;
    table[b'f' as usize] = 15;
    table[b'g' as usize] = 16;
    table[b'h' as usize] = 17;
    table[b'j' as usize] = 18;
    table[b'k' as usize] = 19;
    table[b'm' as usize] = 20;
    table[b'n' as usize] = 21;
    table[b'p' as usize] = 22;
    table[b'q' as usize] = 23;
    table[b'r' as usize] = 24;
    table[b's' as usize] = 25;
    table[b't' as usize] = 26;
    table[b'v' as usize] = 27;
    table[b'w' as usize] = 28;
    table[b'x' as usize] = 29;
    table[b'y' as usize] = 30;
    table[b'z' as usize] = 31;

    table
};

/// Encode 20 bytes (160 bits) to 32 Crockford Base32 characters.
///
/// 32 × 5 = 160 bits — a perfect fit, no spare bits.
pub fn encode(bytes: &[u8; 20]) -> [u8; 32] {
    let mut out = [0u8; 32];

    // Process 5 bytes (40 bits) at a time → 8 base32 chars.
    // 20 bytes = 4 groups of 5 bytes = 4 × 8 = 32 chars.
    for group in 0..4 {
        let base = group * 5;
        let b0 = bytes[base] as u64;
        let b1 = bytes[base + 1] as u64;
        let b2 = bytes[base + 2] as u64;
        let b3 = bytes[base + 3] as u64;
        let b4 = bytes[base + 4] as u64;
        let val = (b0 << 32) | (b1 << 24) | (b2 << 16) | (b3 << 8) | b4;

        let out_base = group * 8;
        out[out_base] = ENCODE_ALPHABET[((val >> 35) & 0x1F) as usize];
        out[out_base + 1] = ENCODE_ALPHABET[((val >> 30) & 0x1F) as usize];
        out[out_base + 2] = ENCODE_ALPHABET[((val >> 25) & 0x1F) as usize];
        out[out_base + 3] = ENCODE_ALPHABET[((val >> 20) & 0x1F) as usize];
        out[out_base + 4] = ENCODE_ALPHABET[((val >> 15) & 0x1F) as usize];
        out[out_base + 5] = ENCODE_ALPHABET[((val >> 10) & 0x1F) as usize];
        out[out_base + 6] = ENCODE_ALPHABET[((val >> 5) & 0x1F) as usize];
        out[out_base + 7] = ENCODE_ALPHABET[(val & 0x1F) as usize];
    }

    out
}

/// Decode 32 Crockford Base32 characters to 20 bytes. Case-insensitive.
pub fn decode(input: &[u8; 32]) -> Result<[u8; 20], BidError> {
    let mut out = [0u8; 20];

    // Process 8 chars (40 bits) at a time → 5 bytes.
    for group in 0..4 {
        let in_base = group * 8;
        let mut val: u64 = 0;

        for i in 0..8 {
            let byte = input[in_base + i];
            if byte >= 128 {
                return Err(BidError::InvalidString(format!(
                    "non-ASCII byte at position {}",
                    in_base + i
                )));
            }
            let bits = DECODE_TABLE[byte as usize];
            if bits == 0xFF {
                return Err(BidError::InvalidString(format!(
                    "invalid character '{}' at position {}",
                    byte as char,
                    in_base + i
                )));
            }
            val = (val << 5) | (bits as u64);
        }

        let out_base = group * 5;
        out[out_base] = (val >> 32) as u8;
        out[out_base + 1] = (val >> 24) as u8;
        out[out_base + 2] = (val >> 16) as u8;
        out[out_base + 3] = (val >> 8) as u8;
        out[out_base + 4] = val as u8;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_zeros() {
        let bytes = [0u8; 20];
        let encoded = encode(&bytes);
        assert_eq!(&encoded, b"00000000000000000000000000000000");
    }

    #[test]
    fn test_encode_max() {
        let bytes = [0xFF; 20];
        let encoded = encode(&bytes);
        assert_eq!(&encoded, b"ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ");
    }

    #[test]
    fn test_roundtrip() {
        let bytes: [u8; 20] = [
            0x01, 0x86, 0xA0, 0x7B, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34,
            0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
        ];
        let encoded = encode(&bytes);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_case_insensitive_decode() {
        let bytes: [u8; 20] = [
            0x01, 0x86, 0xA0, 0x7B, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34,
            0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
        ];
        let encoded = encode(&bytes);
        let lower: [u8; 32] = encoded.map(|b| b.to_ascii_lowercase());
        let decoded = decode(&lower).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_invalid_character() {
        let mut input = [b'0'; 32];
        input[31] = b'I';
        assert!(decode(&input).is_err());

        input[31] = b'L';
        assert!(decode(&input).is_err());

        input[31] = b'O';
        assert!(decode(&input).is_err());

        input[31] = b'U';
        assert!(decode(&input).is_err());
    }

    #[test]
    fn test_all_characters_encode_decode() {
        for (i, &ch) in ENCODE_ALPHABET.iter().enumerate() {
            let bits = DECODE_TABLE[ch as usize];
            assert_eq!(bits as usize, i, "mismatch for character '{}'", ch as char);
        }
    }

    #[test]
    fn test_lexicographic_order_matches_byte_order() {
        let mut a = [0u8; 20];
        let mut b = [0u8; 20];
        a[0] = 0x01;
        b[0] = 0x02;

        let ea = encode(&a);
        let eb = encode(&b);
        assert!(ea < eb, "encoded order must match byte order");
    }
}
