/*
Computes the Longest Common Subsequence using Nakatsu algorithm as outlined in:
https://link.springer.com/article/10.1007/BF00264437

It has the nice property of being efficient if strings are similar to each other
due to its O(n(m-p)) complexity, where n,m are string lengths and p is the length
of the LCS sequence.

The original algorithm, capable of computing all subsequences of maximum length
poses a problem for our case due to it's memory requirements which are quadratic
of the (shorter) string length. As the file size can easily be hundreds of MB,
assuming average chunk size of 8KB the length of the strings (of hashes) can easily
become several thousands, leading to huge memory allocations (millions of words and more)

To overcome this problem, we modify the original algorithm to use the linear space, where
we only hold a vector of the previous computed diagonal, which is enough for running the
recursion. The drawback is that we will only compute on LCS. One possible optimization of
the delta-based file system is inspecting alternative LCSs and choosing the one that
corresponds to the longest total chunks length, so that the amount of data sent over the
network would be minimized (our chunks are of variable length). Having only one LCS makes
this optimization impossible. Still, it seems to be a good compromise.

Note that we use 0-based indices in contrast to the Nakatsu paper, which is 1-based.

TODO: could we inspect chunk size at each diagonal step and choose the most appropriate?
*/

pub(crate) fn lcs_nakatsu<T>(a_string: &[T], b_string: &[T]) -> Vec<T>
where
    T: Ord + Copy + std::fmt::Display,
{
    let a_len = a_string.len();
    let b_len = b_string.len();

    // m_string is shorter of the two (unless they're equal)
    let m_string: &[T];
    let n_string: &[T];
    if a_len <= b_len {
        m_string = &a_string;
        n_string = &b_string;
    } else {
        m_string = &b_string;
        n_string = &a_string;
    }
    let m_len: usize = m_string.len();
    let n_len: usize = n_string.len();

    // we stick to the notation used in the paper, so:
    // sigma - shorter string
    // sigma(i) - i-th character of sigma
    // m - sigma's length
    // sigma(i:m) - trailing substring of sigma starting at i-th character
    // tau - longer string
    // tau(h) - h-th character of tau
    // n - tau's length
    // tau(h:n) - trailing substring of tau starting at h-th character
    // L_i(k) - largest h such that sigma(i:m) and tau(h:n) have LCS of length k

    // allocate vector for storing previous computed diagonal and zero its elements
    // which is a way of saying that those values are undefined and allows the uniform
    // handling
    // these undefined Ls are for cases of L_i(k), where i+k > m (no such LCS exists)
    let diagonal_buf_size: usize = m_len + 1;
    let mut diagonal: Vec<usize> = vec![0; diagonal_buf_size]; // linear space (vector) holding previous computed diagonal

    // TODO: run first two j in separate loop to avoid branching

    let mut lcs: Vec<T> = Vec::with_capacity(m_len);
    let mut lcs_j: usize = 1; // for traversing solution in a down-then-left fashion
    for diagonal_len in (0..diagonal_buf_size).rev() {
        println!("i = diagonal_len = {}", diagonal_len);
        let mut solved: bool = true;
        for j in 1..=diagonal_len {
            // here we compute what the paper refers to as L_i(j) for decreasing i's and 
            // increasing j's (diagonal)
            // we store the computed diagonal values right-aligned in a diagonal buffer
            // to avoid overwriting of the previous row values we may need according to
            // Lemma 3 (step L_i(j) may need L_[i+1](j-1)
            let i = diagonal_len - j + 1;
            let diagonal_buf_index = diagonal_buf_size - diagonal_len + i - 1;
            print!("({},{})->{} ", j, i, diagonal_buf_index);
            // diagonal[diagonal_buf_index] still holds L_[i+1](j) from previous step
            let lower_bound = diagonal[diagonal_buf_index];
            let upper_bound = if j >= 2 && diagonal[diagonal_buf_index+1] != 0 {     // paper calls it range
                diagonal[diagonal_buf_index+1]      // TODO: this may be 0, what then? shouldn't we populate with n_len?
            } else {
                n_len + 1
            };
            print!("searching {} .. {} ", lower_bound, upper_bound);
            // search for character
            let mut l_ij: usize = lower_bound;
            let searched_character = m_string[i-1];
            for h in (lower_bound+1..upper_bound).rev() {
                if n_string[h-1] == searched_character {
                    print!("found at: {} ", h);
                    l_ij = h;
                    if j >= lcs_j {
                        lcs.push(searched_character);
                        lcs_j = j + 1;
                    }
                    break;
                }
            }
            diagonal[diagonal_buf_index] = l_ij;
            print!("L = {}\n", l_ij);
            if l_ij == 0 {
                solved = false;
                break;
            }
        }
        if solved {
            return lcs;
        }
    }
    lcs
}

#[test]
fn test_lcs_nakatsu() {
    /*
         |0|1|2|3|4|5|6|7|
         |b|c|d|a|b|a|b| |
     0 |c| | | | | | | | |
     1 |b| | | | | | | | |
     2 |a| | | | | | | | |
     3 |c| | | | | | | | |
     4 |b| | | | | | | | |
     5 |a| | | | | | | | |
     6 |a| | | | | | | | |
     7 |b| | | | | | | | |
     8 |a| | | | | | | | |
    */
    let a_string = "bcdabab".as_bytes(); // ascii-only so as_bytes is ok
    let b_string = "cbacbaaba".as_bytes();
    let lcs = lcs_nakatsu(a_string, b_string);

    println!("LCS = {}", String::from_utf8(lcs).unwrap());
    assert!(false);
}
