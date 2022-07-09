/* 
    PolynomialSlidingWindowHash 

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
const DEFAULT_BASE: u32 = 29791;      // lower than modulus

// the parameters (modulus, base) are expected to be 32-bit
// we run hashing internally in 64-bit precision to as even 
// single 32-bit operand multiplication could make it overflow
// also, for performance reasons we use signed integers, otherwise
// we'd need to add the modulus prior to modulo operation to avoid
// negative numbers which could result in some wasted cycles

// TODO: we could probably let it overflow but it might adversely
// affect collision rate (just a hypothesis, to be checked)

pub(crate) struct PolynomialSlidingWindowHash {
    modulus: u64,
    base: u64,
    hash: u64,
    buffer: Vec<u8>,        // circular buffer
    buffer_tap: usize,
    buffer_mask: usize,     // for efficient wrapping (provided & is faster than & in Rust)
    max_pow: u64,           // p^(n-1) mod m, precomputed for performance reason
}

impl RollingHash for PolynomialSlidingWindowHash {
    
    fn push(&mut self, byte: u8) {
        let byte_entering_window = u64::from(byte);
        let byte_exiting_window = (u64::from(self.buffer[self.buffer_tap]) * self.max_pow) % self.modulus;
        // stay positive, said rastafaray (although what he meant was: stay unsigned)
        // to do so, we need to add self.modulus prior to subtracting the exiting value
        // and also compute modulus of the (high) exiting value before subtracting it
        // TODO: try running this in signed arithmetic to avoid these steps and profile
        // to see which is faster
        self.hash = ((self.hash + self.modulus - byte_exiting_window) * self.base + byte_entering_window) % self.modulus;
        self.buffer[self.buffer_tap] = byte;
        self.buffer_tap = (self.buffer_tap + 1) & self.buffer_mask;
    }

    fn reset(&mut self) {
        self.hash = 0;
    }

    fn hash(&self) -> u32 {
        self.hash.try_into().unwrap()
    }
}

impl PolynomialSlidingWindowHash {
    
    // window_size must be a power of 2
    pub(crate) fn new(window_size: u32, modulus: Option<u32>, base: Option<u32>) -> Self {
        assert!(is_power_of_two(window_size), "Sliding window size must be power of 2");
               
        let modulus = modulus.unwrap_or(DEFAULT_MODULUS);
        let base = base.unwrap_or(DEFAULT_BASE);

        PolynomialSlidingWindowHash{ 
            modulus: u64::from(modulus),
            base: u64::from(base),
            hash: 0u64,
            buffer: vec![0; usize::try_from(window_size).unwrap()],
            buffer_tap: 0,
            buffer_mask: usize::try_from(window_size-1).unwrap(),
            max_pow: u64::from(mod_power(base, window_size-1, modulus)),
        }
    }
}

#[test]
#[should_panic (expected = r#"Sliding window size must be power of 2"#)]
fn test_polynomial_sliding_window_hash_wrong_window_size() {
    let _ = PolynomialSlidingWindowHash::new(33, Some(1000000007), Some(29791));
}

#[test]
fn test_polynomial_rolling_hash() {
    // trying some basic sequence first
    let mut hasher = PolynomialSlidingWindowHash::new(4, Some(1000), Some(3));
    let input: &[u8] = &[1,2,3,4,5,6];
    hasher.push(input[0]);
    assert_eq!(hasher.hash(), 1);
    hasher.push(input[1]);
    assert_eq!(hasher.hash(), 5);
    hasher.push(input[2]);
    assert_eq!(hasher.hash(), 18);
    hasher.push(input[3]);
    assert_eq!(hasher.hash(), 58);
    hasher.push(input[4]);
    assert_eq!(hasher.hash(), 98);
    hasher.push(input[5]);
    assert_eq!(hasher.hash(), 138);

    // and now some less naive examples
    let mut hasher = PolynomialSlidingWindowHash::new(16, Some(1000000007), Some(29791));

    let input = "equilibrium is a state of no motion";
    for (i, byte) in input.bytes().enumerate() {
        println!("{}, {}", i, byte);
        hasher.push(byte);
    }
    assert_eq!(hasher.hash(), 958536060);

    let input = "standing still is a state of no motion";
    for byte in input.bytes() {
        hasher.push(byte);
    }
    assert_eq!(hasher.hash(), 958536060);

    let input = "eiger is an alpine peak";
    for byte in input.bytes() {
        hasher.push(byte);
    }
    assert_eq!(hasher.hash(), 682459160);

    let input = "that remains in a state of no motion";
    for byte in input.bytes() {
        hasher.push(byte);
    }
    assert_eq!(hasher.hash(), 958536060);
}