/*
    PolynomialRollingHash

    Implements polynomial sliding window rolling hash function of the form:
    hash[x] = [ p^(n-1)*x[0] + p^(n-2)*x[1] + ... + x[n-2]*p + x[n-1] ] mod m

    where:
    x - input data window over which has gets computed (n points)
    n - sliding window size
    m - large prime
    p - integer constant
*/

use super::rolling_hash::*;
use crate::helper::*;

const DEFAULT_MODULUS: u32 = 1000000007;
const DEFAULT_BASE: u32 = 29791; // lower than modulus

// the parameters (modulus, base) are expected to be 32-bit
// we run hashing internally in 64-bit precision to as even
// single 32-bit operand multiplication could make it overflow
// also, for performance reasons we use signed integers, otherwise
// we'd need to add the modulus prior to modulo operation to avoid
// negative numbers which could result in some wasted cycles

// TODO: we could probably let it overflow but it might adversely
// affect collision rate (just a hypothesis, to be checked)

pub(crate) struct PolynomialRollingHash {
    modulus: u64,
    base: u64,
    rolling_hash: u64,
    overall_hash: u64,
    buffer: Vec<u8>, // circular buffer
    buffer_tap: usize,
    buffer_mask: usize, // for efficient wrapping (provided & is faster than % in Rust)
    max_pow: u64,       // p^(n-1) mod m, precomputed for performance reason
}

impl RollingHash for PolynomialRollingHash {

    fn push(&mut self, byte: u8) {
        // here we exploit the modulo-arithmetic identities to stay within range and not 
        // cause overflow; this means some extra % operations so it may actually be more
        // efficient to run it in signed arithmetic
        // (a + b) % m = (a % m + b % m) % m
        // (a * b) % m = (a % m * b % m) % m
        let byte_entering_window = u64::from(byte);
        let byte_exiting_window =
            (u64::from(self.buffer[self.buffer_tap]) * self.max_pow) % self.modulus;
        // stay positive - rastafaray used to say (although what he meant probably was: stay unsigned)
        // to do so, we need to add self.modulus prior to subtracting the exiting value
        // and also compute % of the (high) exiting value before subtracting it
        // this is because Rust % operator returns negative numbers for negative arguments
        // (unlike some other programming languages)
        // TODO: how about running this in signed arithmetic to avoid these steps?
        self.overall_hash = (self.overall_hash * self.base + byte_entering_window) % self.modulus;
        self.rolling_hash = ((self.rolling_hash + self.modulus - byte_exiting_window) * self.base
            + byte_entering_window)
            % self.modulus;
        self.buffer[self.buffer_tap] = byte;
        self.buffer_tap = (self.buffer_tap + 1) & self.buffer_mask;
    }

    fn reset(&mut self) {
        self.overall_hash = 0;
        // no need to reset self.hash or clear the buffer here
        // we won't allow chunk sizes smaller than sliding window size
        // so by the time this min size is reached the old values will get overwritten
    }

    fn get_rolling_hash(&self) -> u32 {
        self.rolling_hash.try_into().unwrap()
    }

    fn get_overall_hash(&self) -> u32 {
        self.overall_hash.try_into().unwrap()
    }

    fn get_window_size(&self) -> usize {
        self.buffer.len()
    }
}

impl PolynomialRollingHash {

    // window_size must be a power of 2
    pub(crate) fn new(window_size: u32, modulus: Option<u32>, base: Option<u32>) -> Self {
        assert!(
            is_power_of_two(window_size),
            "Sliding window size must be power of 2"
        );
        let modulus = modulus.unwrap_or(DEFAULT_MODULUS);
        let base = base.unwrap_or(DEFAULT_BASE);

        PolynomialRollingHash {
            modulus: u64::from(modulus),
            base: u64::from(base),
            rolling_hash: 0u64,
            overall_hash: 0u64,
            buffer: vec![0; usize::try_from(window_size).unwrap()],
            buffer_tap: 0,
            buffer_mask: usize::try_from(window_size - 1).unwrap(),
            max_pow: u64::from(mod_power(base, window_size - 1, modulus)),
        }
    }
}

#[test]
#[should_panic(expected = r#"Sliding window size must be power of 2"#)]
fn test_polynomial_sliding_window_hash_wrong_window_size() {
    let _ = PolynomialRollingHash::new(33, Some(1000000007), Some(29791));
}

#[test]
fn test_polynomial_rolling_hash() {
    // trying some basic sequence first
    let mut hasher = PolynomialRollingHash::new(4, Some(1000), Some(3));
    let input: &[u8] = &[1, 2, 3, 4, 5, 6];
    hasher.push(input[0]);
    assert_eq!(hasher.get_rolling_hash(), 1);
    hasher.push(input[1]);
    assert_eq!(hasher.get_rolling_hash(), 5);
    hasher.push(input[2]);
    assert_eq!(hasher.get_rolling_hash(), 18);
    hasher.push(input[3]);
    assert_eq!(hasher.get_rolling_hash(), 58);
    hasher.push(input[4]);
    assert_eq!(hasher.get_rolling_hash(), 98);
    hasher.push(input[5]);
    assert_eq!(hasher.get_rolling_hash(), 138);

    // and now some less naive examples
    let mut hasher = PolynomialRollingHash::new(16, Some(1000000007), Some(29791));

    let input = "equilibrium is a state of no motion";
    for (i, byte) in input.bytes().enumerate() {
        println!("{}, {}", i, byte);
        hasher.push(byte);
    }
    assert_eq!(hasher.get_rolling_hash(), 958536060);

    let input = "standing still is a state of no motion";
    for byte in input.bytes() {
        hasher.push(byte);
    }
    assert_eq!(hasher.get_rolling_hash(), 958536060);

    let input = "eiger is an alpine peak";
    for byte in input.bytes() {
        hasher.push(byte);
    }
    assert_eq!(hasher.get_rolling_hash(), 682459160);

    let input = "that remains in a state of no motion";
    for byte in input.bytes() {
        hasher.push(byte);
    }
    assert_eq!(hasher.get_rolling_hash(), 958536060);
}
