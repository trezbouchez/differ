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
2. USed generics instead of dependency injection. Algorithms cannot be chosen at runtime. Can be easily changed.
3. There are some hard-coded assumptions about hash types being used (rolling hash u32, proper hash ...). We could abstract them out using traits and generics if necessary.

feeding sequences from disk

TO BE DONE:
DONE: slicer process - there's something not quite right in the process code, the min/max chunk sizes won't be observed
DONE: slicer termination (flushinh when sequence ends so that the last chunk is detected)
DONE: remove computing cheap hash from 
DONE: compute LCS

pretty-print progress
finish Hunt-Szymanski
wrap it all up in a single diff routine (process accepting buffer, finalize)
review TODOs
