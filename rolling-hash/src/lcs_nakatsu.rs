/*
Computes the Longest Common Subsequence using Nakatsu algorithm as outlined in:
https://link.springer.com/article/10.1007/BF00264437

TIME:   O(n(m-p))
SPACE:  O(nm)

where:
n,m - the legths of the inputs
p   - the length of the LCS

It has the desired property of being efficient if inputs are similar which is
the valid assumption for distributed file system (it's actually the motivation
behind it). It approaches linear time if inputs are equal.

The drawback is that it is quadratic space so the allocated memory grows big
for larger inputs.

This implementation only returns one subsequence. If all are necessary, the L 
triangular matrix need to be filled and all traceback paths must be followed.

Possible optimizations:
1. Run first two j's in separate loop to avoid branching
2. Reduce memory requirements by two by smart addressing and only allocating the triangle
3. Use 0-based indices (paper uses 1-based and we sticked to it for legibility)
4. Use binary search when tracing back (horizontally). Not sure it'll help when inputs are similar.
*/

pub(crate) fn lcs_nakatsu<T>(a_string: &[T], b_string: &[T]) -> Vec<T>
where
    T: Ord + Copy,
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

    // TODO: run first two j's in separate loop to avoid branching

    // initialize the L matrix
    let m_size = (m_len + 1) * (m_len + 1);
    let mut l: Vec<usize> = Vec::with_capacity(m_size);
    unsafe { l.set_len(m_size) }; // this is safe, we only need to initialize the diagonal
    let mut i = m_len;
    for _ in 0..m_len + 1 {
        l[i] = 0;
        i += m_len;
    }

    let mut diagonal_len = m_len;
    while diagonal_len > 0 {
        let mut solved: bool = true;
        let mut prev_l = 0; // L_i+1(j-1)
        for j in 1..=diagonal_len {
            let i = diagonal_len - j + 1;
            let index = (j - 1) * (m_len + 1) + i - 1;
            let lower_bound = l[index + 1];
            let upper_bound = if j >= 2 && prev_l != 0 {
                // paper calls it range
                prev_l
            } else {
                n_len + 1
            };
            l[index] = lower_bound;
            let searched_character = m_string[i - 1];
            for h in (lower_bound + 1..upper_bound).rev() {
                if n_string[h - 1] == searched_character {
                    l[index] = h;
                    break;
                }
            }
            prev_l = l[index];
            if l[index] == 0 {
                solved = false;
                break; // go to the next diagonal
            }
        }

        if solved {
            break;
        }

        diagonal_len -= 1;
    }

    for j in 0..m_len + 1 {
        for i in 0..m_len + 1 {
            print!("{},", l[j * (m_len + 1) + i]);
        }
        print!("\n");
    }
    print!("\n");

    // trace back the longest subsequence
    let mut lcs: Vec<T> = Vec::with_capacity(diagonal_len);
    let mut index = (diagonal_len - 1) * (m_len + 1);
    while index > 0 {
        while l[index] == l[index + 1] {
            index += 1;
        }
        lcs.push(n_string[l[index] - 1]);
        index = if index > m_len { index - m_len } else { break };
    }
    lcs
}

#[test]
fn test_lcs_nakatsu() {
    // This is not the most reliable way of testing but it works for this particular implementation
    // We could make the test fail even though the lcs_nakatsu routing were still correct
    // by tracing back the subsequence using an alternative path.
    // This is because there can be multiple solutions sequences to the LCS 
    // A robust test would probably need to list them all and check if at least one matches
    // what the lcs_nakatsu function returns. The problem is that we only ever compute one, so
    // we'll stick to this test for now.

    let a_string = "bcdabab".as_bytes(); // ascii-only so as_bytes is ok
    let b_string = "cbacbaaba".as_bytes();
    let lcs = lcs_nakatsu(a_string, b_string);
    let lcs_string = String::from_utf8(lcs).unwrap();
    assert_eq!(lcs_string, "bcaba");

    let b_string = "equilibrium".as_bytes();
    let a_string = "eiger".as_bytes(); // ascii-only so as_bytes is ok
    let lcs = lcs_nakatsu(a_string, b_string);
    let lcs_string = String::from_utf8(lcs).unwrap();
    assert_eq!(lcs_string, "eir");

    let a_string = "a blockchain is a growing list of records".as_bytes();
    let b_string = "the blockchain - an ever-growing decentralized ledger".as_bytes();
    let lcs = lcs_nakatsu(a_string, b_string);
    let lcs_string = String::from_utf8(lcs).unwrap();
    assert_eq!(lcs_string, " blockchain  a growing li er");
}
