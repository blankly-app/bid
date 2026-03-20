//! Monotonic ID generator with thread-local state.

use crate::bid::Bid;
use crate::error::{BidError, Result};
use std::cell::RefCell;

struct Generator {
    last_ms: u64,
    last_random: [u8; 10],
}

thread_local! {
    static GENERATOR: RefCell<Generator> = const { RefCell::new(Generator {
        last_ms: 0,
        last_random: [0u8; 10],
    }) };
}

/// Generate a new monotonic Bid (tag_hash = 0).
///
/// Within the same millisecond, IDs are guaranteed to be strictly increasing
/// by incrementing the random component. If the clock moves backwards, the
/// last seen timestamp is reused to maintain monotonicity.
pub fn generate() -> Result<Bid> {
    GENERATOR.with(|cell| {
        let mut gen = cell.borrow_mut();
        let now_ms = current_timestamp_ms();

        if now_ms > gen.last_ms {
            let mut random = [0u8; 10];
            getrandom::getrandom(&mut random)
                .map_err(|e| BidError::RandomSource(e.to_string()))?;

            gen.last_ms = now_ms;
            gen.last_random = random;
        } else {
            increment_random(&mut gen.last_random)?;
        }

        Ok(Bid::from_parts(0, gen.last_ms, &gen.last_random))
    })
}

/// Increment a 10-byte (80-bit) big-endian integer by 1.
fn increment_random(random: &mut [u8; 10]) -> Result<()> {
    for byte in random.iter_mut().rev() {
        let (val, overflow) = byte.overflowing_add(1);
        *byte = val;
        if !overflow {
            return Ok(());
        }
    }
    Err(BidError::MonotonicOverflow)
}

fn current_timestamp_ms() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is before Unix epoch")
            .as_millis() as u64
    }

    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_returns_ok() {
        let bid = generate().unwrap();
        assert!(!bid.is_nil());
    }

    #[test]
    fn test_monotonic_ordering() {
        let mut prev = generate().unwrap();
        for _ in 0..1000 {
            let next = generate().unwrap();
            assert!(next > prev, "IDs must be strictly increasing");
            prev = next;
        }
    }

    #[test]
    fn test_timestamp_is_reasonable() {
        let bid = generate().unwrap();
        let ms = bid.timestamp_ms();
        assert!(ms > 1_704_067_200_000);
        assert!(ms < 4_102_444_800_000);
    }

    #[test]
    fn test_increment_random() {
        let mut r = [0u8; 10];
        increment_random(&mut r).unwrap();
        assert_eq!(r, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);

        let mut r = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF];
        increment_random(&mut r).unwrap();
        assert_eq!(r, [0, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
    }

    #[test]
    fn test_increment_random_overflow() {
        let mut r = [0xFF; 10];
        assert!(increment_random(&mut r).is_err());
    }
}
