# rolling-hash
Rolling hash-based file differ

Optimizations to consider:
1. Don't run the rolling hash circular buffer for the first min_chunk_size bytes. This could save some computations for those first steps
2. Choose the best rolling hash implementation and inline its 'push' to avoid function call overhead on a per-byte basis
3. Can (and should) we avoid using Vec in longest_common_subsequence and run it all on stack?
4. Find all LCS and choose the one depending on chunk sizes (longest in terms of bytes, so the least amount of data needs to be sent). In case of Nakatsu algorithm this greatly increases memory requirements.

Other remarks:
1. Some helper functions assume particular types (u8, u32). They could be made more general by using generics so that
we'd have flexibility over choosing hash length etc.
