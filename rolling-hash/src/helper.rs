use std::mem;

// fast way of checking if integer is a power of 2, note it won't work for 0!
pub(crate) fn is_power_of_two(x: u32) -> bool {
    x & (x - 1) == 0
}

// computing u32 power in modular arithmetic without overflow
pub(crate) fn mod_power(base: u32, exponent: u32, modulus: u32) -> u32 {
    if modulus == 1 {
        return 0;
    }
    let mut c = 1u64;
    let base = u64::from(base);
    let exponent = u64::from(exponent);
    let modulus = u64::from(modulus);
    for _ in 0..exponent {
        c = (c * base) % modulus;
    }
    u32::try_from(c).unwrap()
}

#[test]
fn test_is_power_of_two() {
    assert!(is_power_of_two(2048));
    assert!(is_power_of_two(65536));
    assert!(!is_power_of_two(32767));
    assert!(!is_power_of_two(32769));
}

#[test]
fn test_mod_power() {
    assert_eq!(mod_power(2, 12, 13), 1);
    assert_eq!(mod_power(3, 17, 5), 3);
    assert_eq!(mod_power(31, 24, 17), 16);
    assert_eq!(mod_power(31, 64, 1000000007), 822947887);
}