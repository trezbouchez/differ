/*
Computes the Longest Common Subsequence using Hunt-Szymanski algorithm as outlined in:
https://imada.sdu.dk/~rolf/Edu/DM823/E16/HuntSzymanski.pdf
    
TIME:   O((r+m) log n)
SPACE:  O(nm)

where:
n,m - the legths of the inputs
r   - the number of matching character pairs

Works well when inputs similarity is low. Otherwise it approaches O(n^2 logn) which
is even worse than the basic dynamic programming algorithm. 
Doesn't seem to be the best choice for network file system where the motivation is
that files usually only differ slightly.
Also, it is quadratic space so the allocated memory grows big for larger inputs.

This implementation only returns one subsequence.
*/

#[allow(dead_code)]
pub(crate) fn lcs_hunt_szymanski_len<T>(a_string: &[T], b_string: &[T]) -> usize
where
    T: Ord,
{
    // 1. Find coordinates of all pairs with matching characters
    let matching_characters_coords = matching_characters_coordinates(a_string, b_string);

    matching_characters_coords.len()
    // 2. Determine head indices of the last row of dynamic programming matrix
    // let mut head_indices: Vec<usize> = vec![0];
    // let a_string_len = a_string.len();
    // let b_string_len = b_string.len();
    // let matching_characters_index = 
}

// Computes the longest common subsequence
#[allow(dead_code)]
pub(crate) fn lcs_hunt_szymanski<T>(_a_string: &[T], _b_string: &[T])
where
    T: Ord,
{
}

// Returns the coordinates of the matching characters (cartesian product of their indices within the strings)
// This method is faster than checking all cartesian product elements (brute force) and can be done in
// O(r log n + m log(m)) instead of O(n*m)
#[allow(dead_code)]
fn matching_characters_coordinates<T>(a_string: &[T], b_string: &[T]) -> Vec<(usize,usize)>
where
    T: Ord
{
    // annotate characters with their positions within the string  
    let mut a_string: Vec<(&T,usize)> = a_string
        .iter()
        .enumerate()
        .map(|(position, character)| (character, position))
        .collect();
    let mut b_string: Vec<(&T,usize)> = b_string
        .iter()
        .enumerate()
        .map(|(position, character)| (character, position))
        .collect();

    // sort by character
    a_string.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    b_string.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));

    // iterate over matching characters and get cross product (indices of matching characters)
    let mut matching_character_coords: Vec<(usize,usize)> = Vec::new();
    let mut b_index: usize = 0;
    let b_string_len = b_string.len();
    for a in a_string.iter() {
        while b_index < b_string_len && b_string[b_index].0 < a.0 {   // advance b until we get a match
            b_index += 1;
        }
        let b_first_index_matching_a = b_index;
        while b_index < b_string_len && b_string[b_index].0 == a.0 {  // store matching positions pairs
            let b = b_string[b_index];
            matching_character_coords.push((a.1+1, b.1+1));
            b_index += 1;
        }
        b_index = b_first_index_matching_a;                           // restore b_index
    }

    // sort in ascending order on the first coordinate and descending on the other
    matching_character_coords.sort_by(|lhs,rhs| {
        if lhs.0 == rhs.0 {
            rhs.1.cmp(&lhs.1)
        } else {
            lhs.0.cmp(&rhs.0)
        }
    });

    matching_character_coords
}

#[test]
fn test_hunt_szymanski_matching_character_coordinates() {
    let a_string = "EQUILIBRIUM".as_bytes();    // ascii-only so as_bytes is ok
    let b_string = "EIGER".as_bytes();
    let coords = matching_characters_coordinates(a_string, b_string);
    assert_eq!(coords, vec![(1,4),(1,1),(4,2),(6,2),(8,5),(9,2)]);
}

// #[test]
// fn test_longest_common_subsequence() {
//     let a_string = "EQUILIBRIUM".as_bytes();    // ascii-only so as_bytes is ok
//     let b_string = "EIGER".as_bytes();
//     let lcs = lcs_hunt_szymanski_len(a_string, b_string);
//     assert_eq!(lcs, 3);
// }