use crate::delta::*;
use crate::hasher::sha256::*;
// use crate::lcs::hunt_szymanski::*;
use crate::lcs::nakatsu::*;
use crate::rolling_hasher::polynomial::*;
use crate::slicer::*;

const DEFAULT_WINDOW_SIZE: u32 = 1000000007;
const DEFAULT_MIN_CHUNK_SIZE: usize = 4096;
const DEFAULT_MAX_CHUNK_SIZE: usize = 16384;
const DEFAULT_BOUNDARY_MASK: u32 = (1 << 12) - 1; // 12 least significant bits set, avg chunk size is 2^12=4096

/*
    Compares two versions of data buffers or streams and returns delta which
    describes how to patch the old data to become new data, reusing chunks of
    old data whenever possible

    Two ways of computing delta are possible:
    1. Comparing complete in-memory buffers of data. To do so, simply call
       let delta = Differ::diff(...);
       passing the buffers as well as some slicing parameters as arguments

    2. Buffered processing which allows for feeding the Differ instance with
       incoming data and then, once the streams have been read, obtaining delta:
       let mut differ = Differ::new(...);
       differ.process_old(...);
       differ.process_old(...);
       differ.process_new(...);
       differ.process_new(...);
       differ.process_old(...);
       differ.process_new(...);
       let delta = differ.finalize();       // will consume differ

    The code uses Polynomial rolling hash (Rabin-Karp) for slicing streams of data into chunks
    of variable size, which are then hashed with SHA256 and compared using Nakatsu Longest
    Common Subseqence algorithm which is efficient when streams are similar (this seems to
    be a valid assumptions for the application which is a distributed storage system)

    Alternative versions of rolling hash (moving average), digest (SHA1, MD5) and LCS (Hunt-Szymanski)
    are available.
    They cannot be switched at runtime and require the code to be modified.
    The Slicer generic struct is taking RollingHasher and Hasher traits as compile-time arguments.
    To try Hunt-Szymanski LCS (more appropriate when differences are substantial) replace
    lcs_nakatsu function call with lcs_hunt_szymanski.

    Some ideas to consider/explore:

    - implementing Kumar LCS algorithm which is O(n(m-p)) time (like  Nakatsu) but also linear
      space (unlike Nakatsu which is quadratic, what may become a problem for large data)
      https://www.academia.edu/4127816/A_Linear_Space_Algorithm_for_the_LCS_Problem

    - using more efficient rolling hash algorithms, like the Gear used in FastCDC
      https://pdfs.semanticscholar.org/64b5/ce9ff6c7f5396cd1ec6bba8a9f5f27bc8dba.pdf

    - the actual delta file (to be sent over network) should contain OLD/NEW segments, where
      OLD segments only define ranges (client already has the data), while NEW contains actual
      data (client needs it)

    - considering lighter digest algorithms like MD5 (the fact that it's broken shouldn't be a
      problem for this particular application)

    - using more sophisticated slicing to minimize producing chunk of fixed size (max_chunk_size)
      which may result in some boundary-shift issues and thus increased bandwidth (too much of a
      new file being sent over the network); two or more alternative boundary thresholds is one
      idea to explore (to increase probability of boundary detection when chunks size is becoming
      large)
*/

pub struct Differ {
    slicer_old: Slicer<PolynomialRollingHasher, Sha256Hasher>,
    slicer_new: Slicer<PolynomialRollingHasher, Sha256Hasher>,
    is_finalized: bool,
}

impl Differ {
    /// diff
    /// 
    /// Compares two versions of in-memory data (byte) buffers and returns delta
    /// 
    /// Arguments:
    /// buffer_old      - points at the old data buffer
    /// buffer_new      - points at the new (updated) data buffer
    /// window_size     - is rolling hash sliding window size
    /// min_chunk_size  - the minimum chunk size
    /// max_chunk_size  - the maximum chunk size
    /// boundary_mask   - the bit mask used as a threshold for boundary detection
    /// 
    /// Returned:
    /// the vector of Segments which are the byte ranges of the old and new data buffers
    /// that need to be put together to recreate the new updated file
    #[allow(dead_code)]
    pub(crate) fn diff(
        buffer_old: &[u8],
        buffer_new: &[u8],
        window_size: Option<u32>,
        min_chunk_size: Option<usize>,
        max_chunk_size: Option<usize>,
        boundary_mask: Option<u32>,
    ) -> Vec<Segment> {
        let mut differ = Differ::new(window_size, min_chunk_size, max_chunk_size, boundary_mask);

        differ.process_old(buffer_old);
        differ.process_new(buffer_new);

        differ.finalize()
    }

    /// new
    /// 
    /// Creates a new Differ instance to be used with buffered file processing
    /// 
    /// Arguments:
    /// window_size     - is rolling hash sliding window size
    /// min_chunk_size  - the minimum chunk size
    /// max_chunk_size  - the maximum chunk size
    /// boundary_mask   - the bit mask used as a threshold for boundary detection
    /// 
    /// Returned:
    /// the Differ instance
    pub(crate) fn new(
        window_size: Option<u32>,
        min_chunk_size: Option<usize>,
        max_chunk_size: Option<usize>,
        boundary_mask: Option<u32>,
    ) -> Differ {
        let window_size = window_size.unwrap_or(DEFAULT_WINDOW_SIZE);
        let min_chunk_size = min_chunk_size.unwrap_or(DEFAULT_MIN_CHUNK_SIZE);
        let max_chunk_size = max_chunk_size.unwrap_or(DEFAULT_MAX_CHUNK_SIZE);
        let boundary_mask = boundary_mask.unwrap_or(DEFAULT_BOUNDARY_MASK);

        let (slicer_old, slicer_new) =
            make_slicers(window_size, min_chunk_size, max_chunk_size, boundary_mask);

        Differ {
            slicer_old,
            slicer_new,
            is_finalized: false,
        }
    }

    /// process_old, process_new
    /// 
    /// Processes new buffer of the old and new file, respectively. Can be called in
    /// any order, e.g. old and new buffers can be interleaved and processed concurrently
    /// 
    /// Arguments:
    /// buffer          - the buffer of the file to be processed
    pub(crate) fn process_old(&mut self, buffer: &[u8]) {
        assert!(
            !self.is_finalized,
            "Alrady finalized, cannot accept more input."
        );
        self.slicer_old.process(buffer);
    }

    pub(crate) fn process_new(&mut self, buffer: &[u8]) {
        assert!(
            !self.is_finalized,
            "Alrady finalized, cannot accept more input."
        );
        self.slicer_new.process(buffer);
    }

    /// finalize
    /// 
    /// Determines the delta description. To be called once both files have been read.
    /// 
    /// Returned:
    /// the vector of Segments which are the byte ranges of the old and new data buffers
    /// that need to be put together to recreate the new updated file
    pub(crate) fn finalize(mut self) -> Vec<Segment> {
        assert!(!self.is_finalized, "Alrady finalized!");
        self.is_finalized = true;

        let chunks_old = self.slicer_old.finalize();
        let chunks_new = self.slicer_new.finalize();

        // TODO: iterating over chunk arrays (to get vectors of hashes) could be avoided if we
        // introduced a Hashed trait and pass it to LCS routines instead
        let hashes_old: Vec<String> = chunks_old.iter().map(|chunk| chunk.hash.clone()).collect();
        let hashes_new: Vec<String> = chunks_new.iter().map(|chunk| chunk.hash.clone()).collect();

        let lcs = lcs_nakatsu(&hashes_old[..], &hashes_new[..]);
        // let lcs = lcs_hunt_szymanski(&hashes_old[..], &hashes_new[..]);

        delta(&chunks_old, &chunks_new, &lcs[..])
    }
}

fn make_slicers(
    window_size: u32,
    min_chunk_size: usize,
    max_chunk_size: usize,
    boundary_mask: u32,
) -> (
    Slicer<PolynomialRollingHasher, Sha256Hasher>,
    Slicer<PolynomialRollingHasher, Sha256Hasher>,
) {
    let rolling_hasher_old = PolynomialRollingHasher::new(window_size, None, None);
    let hasher_old = Sha256Hasher::new(max_chunk_size);
    let slicer_old = Slicer::new(
        rolling_hasher_old,
        hasher_old,
        boundary_mask,
        min_chunk_size,
        max_chunk_size,
    );

    let rolling_hasher_new = PolynomialRollingHasher::new(window_size, None, None);
    let hasher_new = Sha256Hasher::new(max_chunk_size);
    let slicer_new = Slicer::new(
        rolling_hasher_new,
        hasher_new,
        boundary_mask,
        min_chunk_size,
        max_chunk_size,
    );

    (slicer_old, slicer_new)
}

#[cfg(test)]
mod tests {
    use super::Differ;
    use crate::delta::Segment;
    use crate::reader::read_file;
    use crate::patcher::patch;
    use sha2::{Sha256, Digest};
    use std::{fs::{File/*,remove_file*/}, io::{copy}};

    #[test]
    fn test_differ_data() {
        let old_string = "What a a year in the blockchain sphere. It's also been quite a year for Equilibrium and I thought I'd recap everything that has happened in the company.";
        let new_string = "It's been a year in the blockchain sphere. It's also been quite a year for Equilibrium. I thought I'd recap everything that has happened in the company with a Year In Review post.";

        // avg chunk size 16
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
        let mut patched_string = String::from("");
        for segment in segments {
            patched_string += match segment {
                Segment::Old(range) => &old_string[range],
                Segment::New(range) => &new_string[range],
            };
        }
        assert_eq!(new_string, patched_string);

        // avg chunk size 8
        let window_size: u32 = 4;
        let min_chunk_size: usize = 4;
        let max_chunk_size: usize = 16;
        let boundary_mask: u32 = (1 << 3) - 1; // avg chunk size is 2^3 = 8
        let segments = Differ::diff(
            old_string.as_bytes(),
            new_string.as_bytes(),
            Some(window_size),
            Some(min_chunk_size),
            Some(max_chunk_size),
            Some(boundary_mask),
        );
        let mut patched_string = String::from("");
        for segment in segments {
            patched_string += match segment {
                Segment::Old(range) => &old_string[range],
                Segment::New(range) => &new_string[range],
            };
        }
        assert_eq!(new_string, patched_string);
    }

    #[test]
    fn test_differ_files() -> std::io::Result<()> {
        // avg chunk size 16
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
        
        // process old and new files
        let old_file_path = "../monkey_before.tiff";
        let new_file_path = "../monkey_after.tiff";

        read_file(old_file_path, |bytes, _| {
            differ.process_old(bytes);
        });
        read_file(new_file_path, |bytes, _| {
            differ.process_new(bytes);
        });

        // compute delta
        let segments = differ.finalize();

        // build patched file
        let patched_file_path = "../monkey_patched.tiff";
        let (_old_bytes_used, _new_bytes_used) = patch(old_file_path, new_file_path, patched_file_path, segments)?;

        // println!("Bytes reused: {}", _old_bytes_used);
        // println!("Bytes transferred: {}", _new_bytes_used);

        // compare new and patched
        let mut hasher = Sha256::new();
        let mut new_file = File::open(new_file_path)?;
        _ = copy(&mut new_file, &mut hasher)?;
        let new_hash_bytes = hasher.finalize();

        let mut hasher = Sha256::new();
        let mut patched_file = File::open(new_file_path)?;
        _ = copy(&mut patched_file, &mut hasher)?;
        let patched_hash_bytes = hasher.finalize();

        assert_eq!(new_hash_bytes, patched_hash_bytes);

        // leaving the patched file there so that it can be inspected
        // remove_file(patched_file_path)?;

        Ok(())
    }
}
