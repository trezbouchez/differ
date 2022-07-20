/*
    MovingSumRollingHasher

    Implements very simple moving sum (sliding window) rolling hash
    The hash is computed using modulo 2^32 by simply letting it overflow
*/

use super::rolling_hasher::*;
use crate::helper::*;

pub(crate) struct MovingSumRollingHasher {
    rolling_hash: u32,
    buffer: Vec<u8>, // circular buffer
    buffer_tap: usize,
    buffer_mask: usize, // for efficient wrapping (provided & is faster than % in Rust)
}

impl RollingHasher for MovingSumRollingHasher {
    #[inline(always)]
    fn push(&mut self, byte: u8) -> u32 {
        let byte_entering_window = u32::from(byte);
        let byte_exiting_window = u32::from(self.buffer[self.buffer_tap]);
        self.rolling_hash = self
            .rolling_hash
            .overflowing_add(byte_entering_window)
            .0
            .overflowing_sub(byte_exiting_window)
            .0;
        self.buffer[self.buffer_tap] = byte;
        self.buffer_tap = (self.buffer_tap + 1) & self.buffer_mask;

        self.rolling_hash
    }

    fn get_window_size(&self) -> usize {
        self.buffer.len()
    }
}

impl MovingSumRollingHasher {
    // window_size must be a power of 2
    #[allow(dead_code)]
    pub(crate) fn new(window_size: u32) -> Self {
        assert!(
            is_power_of_two(window_size),
            "Sliding window size must be power of 2"
        );
        MovingSumRollingHasher {
            rolling_hash: 0u32,
            buffer: vec![0; usize::try_from(window_size).unwrap()],
            buffer_tap: 0,
            buffer_mask: usize::try_from(window_size - 1).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = r#"Sliding window size must be power of 2"#)]
    fn test_moving_sum_rolling_hash_wrong_window_size() {
        let _ = MovingSumRollingHasher::new(31);
    }

    #[test]
    fn test_moving_sum_rolling_hash() {
        // trying some basic sequence first
        let mut hasher = MovingSumRollingHasher::new(4);
        let input: &[u8] = &[1, 2, 3, 4, 5, 6];
        assert_eq!(hasher.push(input[0]), 1);
        assert_eq!(hasher.push(input[1]), 3);
        assert_eq!(hasher.push(input[2]), 6);
        assert_eq!(hasher.push(input[3]), 10);
        assert_eq!(hasher.push(input[4]), 14);
        assert_eq!(hasher.push(input[5]), 18);

        // and now some less naive example
        let mut hasher = MovingSumRollingHasher::new(16);
        let input = "equilibrium is a state of no motion";
        let mut hash = 0u32;
        for byte in input.bytes() {
            hash = hasher.push(byte);
        }
        assert_eq!(hash, 1506);

        // check it with overflow
        let mut hasher = MovingSumRollingHasher::new(2);
        hasher.buffer = vec![8, 1];
        let input: &[u8] = &[1, 2, 8];
        assert_eq!(hasher.push(input[0]), u32::MAX - 6); // 1-8 = -7
        assert_eq!(hasher.push(input[1]), u32::MAX - 5); // -7-1+2= -6
        assert_eq!(hasher.push(input[2]), 1); // -6-1+8= 1
    }
}
