/*
    Implements the efficient way of finding the longest common subsequence with
    Hunt-Szymanski algorithm, as described in:
    https://imada.sdu.dk/~rolf/Edu/DM823/E16/HuntSzymanski.pdf
    
    In our case the alphabet characters are hashes (computed for chunks) but we'll
    use the terms "string", "character", "alphabet" as in the aforementioned doc. 

    Hunt-Szymanski computes the common subsequence length in O((n+r) log n)
    This means it is good when number of matching characters (r) is small.
    In the extreme case of identical strings this becomes O((n+n^2) log n) which is worse
    than dynamic programming approach. Depending on the a'priori knowledge about the case
    (low-bandwidth file system), in particular the average extent of the modifications applied
    to the files being compared, we should choose one or another.
    
    In case of a low-bandwidth delta-based file system it may also be beneficial to use
    Hirschberg's algorithm which is O(pn), where p is the LCS length. This yields O(n^2) in the 
    worst case of identical files:
    https://imada.sdu.dk/~rolf/Edu/DM823/E16/Hirschberg.pdf

    Yet another option to research is Crochemore and others:
    https://kclpure.kcl.ac.uk/portal/en/publications/a-fast-and-practical-bitvector-algorithm-for-the-longest-common-subsequence-problem(fdf12a5f-7516-40b4-85bc-41d5082a4aa3).html

    The most promising seems this, though:
    https://link.springer.com/article/10.1007/BF00264437

    https://dml.cz/bitstream/handle/10338.dmlcz/135445/Kybernetika_38-2002-1_3.pdf
    
    This doc, describing bit-vector approach to speeding up LCS calculation is probably worth reading as well.
    Not sure it can help our case as it seems to require allocating arrays for the entire alphabet (hash space).
    https://www.researchgate.net/publication/3940105_Speeding-up_Hirschberg_and_Hunt-Szymanski_LCS_Algorithms
*/

// Computes the longest common subsequence length
pub(crate) fn lcs_len<T>(a_string: &[T], b_string: &[T])
where
    T: Ord,
{
    // 1. Find coordinates of all pairs with matching characters
    let matching_characters_coords = matching_characters_coordinates(a_string, b_string);

    // 2. Determine head indices of the last row of dynamic programming matrix
    let mut head_indices: Vec<usize> = vec![0];
    let a_string_len = a_string.len();
    let b_string_len = b_string.len();
    let mut matching_characters_index = 0;
}

// Computes the longest common subsequence
pub(crate) fn lcs<T>(a_string: &[T], b_string: &[T])
where
    T: Ord,
{
}

// Returns the coordinates of the matching characters (cartesian product of their indices within the strings)
// This method is faster than checking all cartesian product elements (brute force) and can be done in
// O(r log n + m log(m)) instead of O(n*m)
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
fn test_matching_characters_coordinates() {
    /*
          |0|1|2|3|4|5|
          |-|E|I|G|E|R|
      0 |-| | | | | | |
      1 |E| |X| | |X| | 
      2 |Q| | | | | | |
      3 |U| | | | | | |
      4 |I| |X| | | | |
      5 |L| | | | | | |
      6 |I| | |X| | | |
      7 |B| | | | | | |
      8 |R| | | | | |X|
      9 |I| |X| | | | |
     10 |U| | | | | | |
     11 |M| | | | | | |
     */
    let a_string = "EQUILIBRIUM".as_bytes();    // ascii-only so as_bytes is ok
    let b_string = "EIGER".as_bytes();
    let matching_character_coords = matching_characters_coordinates(&a_string, &b_string);
    assert_eq!(matching_character_coords, vec![(1,4),(1,1),(4,2),(6,2),(8,5),(9,2)]);
}

#[test]
fn test_longest_common_subsequence() {
    let a_string = "EQUILIBRIUM".as_bytes();    // ascii-only so as_bytes is ok
    let b_string = "EIGER".as_bytes();
    let lcs = lcs_len(a_string, b_string);
    // assert_eq!(lcs, 3);
}