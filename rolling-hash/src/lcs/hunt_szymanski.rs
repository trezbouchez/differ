/*
Computes the Longest Common Subsequence using Hunt-Szymanski algorithm as proposed in:
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
This is the algorithm used by Linux diff.

This implementation only returns one subsequence.

Possible optimizations:
1. Improve the way new nodes are determined for each row. Now it's done by comparing rows of
   head_indices. It should be done with an unordered set data structure (hash table?) where 
   moved head_indices are put into (and updated) while the inner while loop is running. Then
   when the loop completes, these indices can be used to insert new nodes.
2. Searching for next active node while backtracing should be made better, at the moment it
   simply checks the nodes one-by-one.
*/

use crate::helper::*;

// Computes the longest common subsequence
#[allow(dead_code)]
pub(crate) fn lcs_hunt_szymanski<T>(a_string: &[T], b_string: &[T]) -> Vec<T>
where
    T: Ord + Clone,
{
    // 1. Find coordinates of all pairs with matching characters
    let r = matching_characters_coordinates(a_string, b_string);
    let r_len = r.len();

    // 2. Determine head indices of dynamic programming matrix and store node coordinates
    let mut head_indices: Vec<usize> = Vec::with_capacity(a_string.len());
    head_indices.push(0);

    let a_len = a_string.len();
    let mut r_index: usize = 0;
    let mut nodes: Vec<(usize, usize, usize)> = vec![(0, 0, 0)]; // row, col, block_index
    let mut prev_row_head_indices: Vec<usize>;
    for i in 1..=a_len {
        // TODO: this is needed to store nodes while we run the algorithm; it should be
        // replaced with more optimized approach using HashSet or similar data structure for
        // storing unique values - we'd store all head_indices that changed over the course of
        // the inner (while) loop and then add nodes when the loop completes
        prev_row_head_indices = head_indices.clone();

        // here we drop the r's for already-processed rows and only perform binary
        // search (lower_bound) within the remaining r's
        // for efficient backtracing, we store nodes, as outlined in the original paper
        let trailing_r = &r[r_index..r_len];

        r_index += lower_bound_by(trailing_r, |lhs| lhs.0.cmp(&i)).unwrap_or(0);
        // iterate over all active indices (matching char pairs) for this row
        while r_index != r_len && r[r_index].0 == i {
            let j = r[r_index].1;
            if let Some(successor) = lower_bound(j, &head_indices) {
                head_indices[successor] = j;
            } else {
                head_indices.push(j);
            }
            r_index += 1;
        }

        // determines nodes
        let block_count = head_indices.len();
        let prev_node_block_count = prev_row_head_indices.len(); 
        for block_index in 0..block_count {
            if block_index == prev_node_block_count || 
            head_indices[block_index] != prev_row_head_indices[block_index] {
                nodes.push((i, head_indices[block_index], block_index));
            }
        }
    }

    // 3. Trace back the subsequence
    let mut active_node_index = nodes.len() - 1;
    let mut char_indices: Vec<usize> = Vec::new();
    while active_node_index > 0 {
        let active_node = nodes[active_node_index];
        char_indices.push(active_node.1);
        let next_active_block_index = active_node.2 - 1;
        let mut node_index = active_node_index - 1;
        while node_index > 0 {
            let node = nodes[node_index];
            if node.0 < active_node.0 && node.1 < active_node.1 && node.2 == next_active_block_index {
                break
            }
            node_index -=1;
        }
        active_node_index = node_index;
    } 

    let mut lcs: Vec<T> = Vec::with_capacity(nodes.len());
    for char_index in char_indices.iter().rev() {
        lcs.push(b_string[char_index-1].clone());
    }

    lcs
}

// Returns the coordinates of the matching characters (cartesian product of their indices within the strings)
// This method is faster than checking all cartesian product elements (brute force) and can be done in
// O(r log n + m log(m)) instead of O(n*m)
#[allow(dead_code)]
fn matching_characters_coordinates<T>(a_string: &[T], b_string: &[T]) -> Vec<(usize, usize)>
where
    T: Ord,
{
    // annotate characters with their positions within the string
    let mut a_string: Vec<(&T, usize)> = a_string
        .iter()
        .enumerate()
        .map(|(position, character)| (character, position))
        .collect();
    let mut b_string: Vec<(&T, usize)> = b_string
        .iter()
        .enumerate()
        .map(|(position, character)| (character, position))
        .collect();

    // sort by character
    a_string.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    b_string.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));

    // iterate over matching characters and get cross product (indices of matching characters)
    let mut matching_character_coords: Vec<(usize, usize)> = Vec::new();
    let mut b_index: usize = 0;
    let b_string_len = b_string.len();
    for a in a_string.iter() {
        while b_index < b_string_len && b_string[b_index].0 < a.0 {
            // advance b until we get a match
            b_index += 1;
        }
        let b_first_index_matching_a = b_index;
        while b_index < b_string_len && b_string[b_index].0 == a.0 {
            // store matching positions pairs
            let b = b_string[b_index];
            matching_character_coords.push((a.1 + 1, b.1 + 1));
            b_index += 1;
        }
        b_index = b_first_index_matching_a; // restore b_index
    }

    // sort in ascending order on the first coordinate and descending on the other
    matching_character_coords.sort_by(|lhs, rhs| {
        if lhs.0 == rhs.0 {
            rhs.1.cmp(&lhs.1)
        } else {
            lhs.0.cmp(&rhs.0)
        }
    });

    matching_character_coords
}

#[test]
fn test_lcs_hunt_szymanski_matching_character_coordinates() {
    let a_string = "EQUILIBRIUM".as_bytes(); // ascii-only so as_bytes is ok
    let b_string = "EIGER".as_bytes();
    let coords = matching_characters_coordinates(a_string, b_string);
    assert_eq!(coords, vec![(1, 4), (1, 1), (4, 2), (6, 2), (8, 5), (9, 2)]);
}

#[test]
fn test_lcs_hunt_szymanski() {
    let a_string = "bcdabab".as_bytes(); // ascii-only so as_bytes is ok
    let b_string = "cbacbaaba".as_bytes();
    let lcs = lcs_hunt_szymanski(a_string, b_string);
    let lcs_string = String::from_utf8(lcs).unwrap();
    assert_eq!(lcs_string, "cabab");

    let a_string = "equilibrium".as_bytes();
    let b_string = "eiger".as_bytes();
    let lcs = lcs_hunt_szymanski(a_string, b_string);
    let lcs_string = String::from_utf8(lcs).unwrap();
    assert_eq!(lcs_string, "eir");

    let a_string = "a blockchain is a growing list of records".as_bytes();
    let b_string = "the blockchain - an ever-growing decentralized ledger".as_bytes();
    let lcs = lcs_hunt_szymanski(a_string, b_string);
    let lcs_string = String::from_utf8(lcs).unwrap();
    assert_eq!(lcs_string, " blockchain  a growing li ed");
}
