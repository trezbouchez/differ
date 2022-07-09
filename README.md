# rolling-hash
Rolling hash-based file differ

Optimizations to consider:
1. Don't run the rolling hash circular buffer for the first min_chunk_size bytes. This could save some computations for those first steps
2. Choose the best rolling hash implementation and inline its 'push' to avoid function call overhead on a per-byte basis
