# differ

File differ and patcher. 

Compares two versions of the file and creates a delta - the recipe for recreating the new, updated version of a file while trying to reuse as much of the old file (the file a client already has) as possible.

The default algorithm uses:
- polynomial rolling-hash algorithm (aka Rabin-Karp) for content-based chunking
- sha256 digest for hashing chunks
- Nakatsu longest common subsequence algorithm (efficient when differences between files are small)

There are some alternative algorithmic blocks included in the code which are not used by the built binary:
- moving sum rolling-hash
- Hunt-Szymanski LCS (good when difference between files is substantial)
- md5, sha1 digest

It's not possible to switch them at runtime - they require (simple) code modifications.

The created delta file is just a simple description. It does not contain any chunk data. To be used in a distributed
storage system the patch file would need to be built containing ranges of the old file to be reused and chunks of
new data to be inserted.

# dependencies

The only external dependencies are `md5`, `sha1` and `sha2` hash crates. Everything else was written from scratch based on the papers (cited in respective files). Because the purpose of this project is the recruitment process, all building blocks were written by the author, including those which could benefit from using easily obtainable data structures or algorithm implementations.

# building and testing

To create the `differ` executable run:
```
cargo build
```
or, for release optimized build:
```
cargo build --release
```


# testing

Unit tests can be run with:
```
cargo test
```

# using

The `differ` executable can be used with:

```
differ <old_file> <new_file> <patched_file> <delta_file>

where:
old_file     - path to the original (old) file
new_file     - path to the updated (new) file
patched_file - patched file will be created at this path (the file recreated from old/new/delta)
delta_file   - delta text file will be create at this location (contains description of all edits performed to build the patched_file)
```

# example

The `example` folder contains one simple example. Running the `example.sh` bash script will build the project and run it. It uses the assets included in the same folder.

# code organization

The main top-level routines are contained in the 'differ.rs' file. They allow for processing in-memory data (buffers containing complete data) and for buffered processing (for large files which won't fit into memory).

In-memory data processing example:
```
let old_string = "What a a year in the blockchain sphere. It's also been quite a year for Equilibrium and I thought I'd recap everything that has happened in the company.";
let new_string = "It's been a year in the blockchain sphere. It's also been quite a year for Equilibrium. I thought I'd recap everything that has happened in the company with a Year In Review post.";
let window_size: u32 = 8;
let min_chunk_size: usize = 8;
let max_chunk_size: usize = 32;
let boundary_mask: u32 = (1 << 4) - 1; // avg chunk size is 2^4 = 16
let segments = Differ::diff(
    old_string.as_bytes(),
    new_string.as_bytes(),
    Some(window_size),
    Some(min_chunk_size),
    Some(max_chunk_size),
    Some(boundary_mask),
);
```

Buffered processing example:
```
let window_size: u32 = 64;
let min_chunk_size: usize = 2048;
let max_chunk_size: usize = 8192;
let boundary_mask: u32 = (1 << 12) - 1; // avg chunk size is 2^12 = 4096
let mut differ = Differ::new(
    Some(window_size),
    Some(min_chunk_size),
    Some(max_chunk_size),
    Some(boundary_mask),
);       
differ.process_old(...);
differ.process_old(...);
differ.process_new(...);
differ.process_new(...);
differ.process_old(...);
differ.process_new(...);
let delta = differ.finalize();       // will consume differ instance
```

Please refer to the unit tests contained in differ.rs file for more details.

# suggested further effort

- implementing Kumar LCS algorithm which is O(n(m-p)) time (like  Nakatsu) but also linear
  space (unlike Nakatsu which is quadratic, what may become a problem for large data:

  https://www.academia.edu/4127816/A_Linear_Space_Algorithm_for_the_LCS_Problem

- using more efficient rolling hash algorithms, like the Gear used in FastCDC:
  
  https://pdfs.semanticscholar.org/64b5/ce9ff6c7f5396cd1ec6bba8a9f5f27bc8dba.pdf

- using more sophisticated slicing to minimize producing chunk of fixed size (max_chunk_size) 
  which may result in some boundary-shift issues and thus increased bandwidth (too much of a
  new file being sent over the network); two or more alternative boundary thresholds is one
  idea to explore (to increase probability of boundary detection when chunks size is becoming
  large)
