use std::cmp::Ordering;

// fast way of checking if integer is a power of 2, note it won't work for 0!
#[allow(dead_code)]
pub(crate) fn is_power_of_two(x: u32) -> bool {
    x & (x - 1) == 0
}

// computing u32 power in modular arithmetic without overflow
#[allow(dead_code)]
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

// performs binary search operations, if the searched item appears multiple times in
// slice, any of the matching indices will be returned
#[allow(dead_code)]
pub(crate) fn binary_search<T>(searched_item: T, sorted_items: &[T]) -> Option<usize>
where
    T: Ord,
{
    binary_search_by(sorted_items, |lhs| lhs.cmp(&searched_item))
}

#[allow(dead_code)]
pub(crate) fn binary_search_by<T, F>(sorted_items: &[T], compare: F) -> Option<usize>
where
    F: Fn(&T) -> Ordering,
{
    let mut low: usize = 0;
    let mut high: usize = sorted_items.len();
    while low < high {
        let mid = (high + low) / 2;
        let mid_item = &sorted_items[mid];
        match compare(mid_item) {
            Ordering::Equal => return Some(mid),
            Ordering::Greater => high = mid,
            Ordering::Less => low = mid + 1,
        }
    }
    None
}

// returns the lowest index for which 'sorted_items[index] >= item' condition holds
// or None if all sorted_items < item
#[allow(dead_code)]
pub(crate) fn lower_bound<T>(item: T, sorted_items: &[T]) -> Option<usize>
where
    T: Ord,
{
    lower_bound_by(sorted_items, |lhs| lhs.cmp(&item))
}

#[allow(dead_code)]
pub(crate) fn lower_bound_by<T, F>(sorted_items: &[T], compare: F) -> Option<usize>
where
    F: Fn(&T) -> Ordering,
{
    let len = sorted_items.len();
    let mut low: usize = 0;
    let mut high: usize = len;
    while low < high {
        let mid = (high + low) / 2;
        let mid_item = &sorted_items[mid];
        match compare(mid_item) {
            Ordering::Less => low = mid + 1,
            _ => high = mid,
        }
    }
    if low == len {
        None
    } else {
        Some(low)
    }
}

// returns the lowest index for which 'sorted_items[index] > item' condition holds
// or None if all sorted_items are <= item
#[allow(dead_code)]
pub(crate) fn upper_bound<T>(item: T, sorted_items: &[T]) -> Option<usize>
where
    T: Ord,
{
    upper_bound_by(sorted_items, |lhs| lhs.cmp(&item))
}

#[allow(dead_code)]
pub(crate) fn upper_bound_by<T, F>(sorted_items: &[T], compare: F) -> Option<usize>
where
    F: Fn(&T) -> Ordering,
{
    let len = sorted_items.len();
    let mut low: usize = 0;
    let mut high: usize = len;
    while low < high {
        let mid = (high + low) / 2;
        let mid_item = &sorted_items[mid];
        match compare(mid_item) {
            Ordering::Greater => high = mid,
            _ => low = mid + 1,
        }
    }
    if low == len {
        None
    } else {
        Some(low)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_binary_search() {
        let sorted_items: &[u8] = &[14, 16, 21, 32, 65, 122, 123, 156];

        let index_of_10 = binary_search(10, sorted_items);
        assert_eq!(index_of_10, None);

        let index_of_14 = binary_search(14, sorted_items);
        assert_eq!(index_of_14, Some(0));

        let index_of_21 = binary_search(21, sorted_items);
        assert_eq!(index_of_21, Some(2));

        let index_of_30 = binary_search(30, sorted_items);
        assert_eq!(index_of_30, None);

        let index_of_122 = binary_search(122, sorted_items);
        assert_eq!(index_of_122, Some(5));

        let index_of_156 = binary_search(156, sorted_items);
        assert_eq!(index_of_156, Some(7));

        let index_of_180 = binary_search(180, sorted_items);
        assert_eq!(index_of_180, None);
    }

    #[test]
    fn test_lower_bound() {
        let sorted_items: &[u8] = &[14, 15, 15, 15, 65, 122, 122, 135, 135, 135];

        let index_of_1 = lower_bound(1, sorted_items);
        assert_eq!(index_of_1, Some(0));

        let index_of_10 = lower_bound(10, sorted_items);
        assert_eq!(index_of_10, Some(0));

        let index_of_14 = lower_bound(14, sorted_items);
        assert_eq!(index_of_14, Some(0));

        let index_of_15 = lower_bound(15, sorted_items);
        assert_eq!(index_of_15, Some(1));

        let index_of_16 = lower_bound(16, sorted_items);
        assert_eq!(index_of_16, Some(4));

        let index_of_122 = lower_bound(122, sorted_items);
        assert_eq!(index_of_122, Some(5));

        let index_of_135 = lower_bound(135, sorted_items);
        assert_eq!(index_of_135, Some(7));

        let index_of_136 = lower_bound(136, sorted_items);
        assert_eq!(index_of_136, None);

        let index_of_200 = lower_bound(200, sorted_items);
        assert_eq!(index_of_200, None);
    }

    #[test]
    fn test_upper_bound() {
        let sorted_items: &[u8] = &[14, 15, 15, 15, 65, 122, 122, 135, 135, 135];

        let index_of_1 = upper_bound(1, sorted_items);
        assert_eq!(index_of_1, Some(0));

        let index_of_10 = upper_bound(10, sorted_items);
        assert_eq!(index_of_10, Some(0));

        let index_of_14 = upper_bound(14, sorted_items);
        assert_eq!(index_of_14, Some(1));

        let index_of_15 = upper_bound(15, sorted_items);
        assert_eq!(index_of_15, Some(4));

        let index_of_16 = upper_bound(16, sorted_items);
        assert_eq!(index_of_16, Some(4));

        let index_of_122 = upper_bound(122, sorted_items);
        assert_eq!(index_of_122, Some(7));

        let index_of_135 = upper_bound(135, sorted_items);
        assert_eq!(index_of_135, None);

        let index_of_200 = upper_bound(200, sorted_items);
        assert_eq!(index_of_200, None);
    }
}
