/*
    The Longest Common Subsequence algorithms researched include:

    Dynamic Programming:
    https://en.wikipedia.org/wiki/Longest_common_subsequence_problem#Code_for_the_dynamic_programming_solution:~:text=Code-,for,-the%20dynamic%20programming%20solution%5B
    TIME:   O(mn)
    SPACE:  O(mn)
    where:
    n,m - the legths of the inputs

    Hunt-Szymanski (implemented):
    https://imada.sdu.dk/~rolf/Edu/DM823/E16/HuntSzymanski.pdf
    TIME:   O((r+m) log n)
    SPACE:  O(r+n)
    where:
    n,m - the legths of the inputs
    r   - the number of matching character pairs
    
    Nakatsu (implemented):
    https://link.springer.com/article/10.1007/BF00264437
    TIME:   O(n(m-p))
    SPACE:  O(nm)
    where:
    n,m - the legths of the inputs
    p   - the length of the LCS

    Hirschberg:
    https://www.ics.uci.edu/~dan/pubs/p664-hirschberg.pdf
    Paper outlines two algorithms:
    Hirschberg #1:      TIME:   O(pn + n log n)
    Hirschberg #2:      TIME:   O(p(m + 1 - p)log n)
    where:
    n,m - the legths of the inputs
    p   - the length of the LCS sequence


    Another very promising algorithm that seems to offer the best properties is.
    It's only been discovered recently and has not been implemented yet.
    
    Kumar:
    https://www.academia.edu/4127816/A_Linear_Space_Algorithm_for_the_LCS_Problem
    TIME:   O(n(m-p))
    SPACE:  O(n)

    This doc, describing bit-vector approach to speeding up LCS calculation is probably worth reading as well.
    Not sure it can help our case as it seems to require allocating arrays for the entire alphabet (hash space).
    https://www.researchgate.net/publication/3940105_Speeding-up_Hirschberg_and_Hunt-Szymanski_LCS_Algorithms

    Returning all solution subsequences (there can be many) would potentially allow us to check the preferred one
    based on the chunk size (to minimize the amount of data sent over the network) but it's not sure whether the
    pros (bandwidth reduction) outweigh the cons (more computations).
*/

pub(crate) trait LCS {
    fn lcs<T: Ord + Clone>(a_string: &[T], b_string: &[T]) -> Vec<T>;
}
