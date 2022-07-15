# rolling-hash
Rolling hash-based file differ

Optimizations to consider:
1. Don't run the rolling hash circular buffer for the first min_chunk_size bytes. This could save some computations for those first steps
2. Choose the best rolling hash implementation and inline its 'push' to avoid function call overhead on a per-byte basis
3. Can (and should) we avoid using Vec in longest_common_subsequence and run it all on stack?
4. Find all LCS and choose the one depending on chunk sizes (longest in terms of bytes, so the least amount of data needs to be sent). In case of Nakatsu algorithm this greatly increases memory requirements.
5. Binary searching rows when tracing back Nakatsu LCS


Other remarks:
1. Some helper functions assume particular types (u8, u32). They could be made more general by using generics so that
we'd have flexibility over choosing hash length etc.
2. USed generics instead of dependency injection. Algorithms cannot be chosen at runtime. Can be easily changed.
3. There are some hard-coded assumptions about hash types being used (rolling hash u32, proper hash ...). We could abstract them out using traits and generics if necessary.
Not sure if main meets the requirement saying that: "Hashing function gets the data as a parameter. Separate possible filesystem operations". The hashing function itself takes data. However, hash calculation needs to be buffered for big
files so it doesn't make sense to pass the complete data (inputs).


feeding sequences from disk

TO BE DONE:
DONE: slicer process - there's something not quite right in the process code, the min/max chunk sizes won't be observed
DONE: slicer termination (flushinh when sequence ends so that the last chunk is detected)
DONE: remove computing cheap hash from 
DONE: compute LCS
DONE: unsafe in Nakatsu causes problems
DONE: pretty-print progress

finish Hunt-Szymanski
wrap it all up in a single diff routine (process accepting buffer, finalize)
review TODOs
test with moving_average
other rolling hash algos (research)
think about how to use bit-wise to optimize certain routines