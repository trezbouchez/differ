use super::rolling_hasher::rolling_hasher::*;
use super::hasher::hasher::*;

pub(crate) struct Slicer<RH: RollingHasher, H: Hasher>
{
    rolling_hasher: RH,
    hasher: H,
    boundary_mask: u32, // if masked hash bits are all zeros, it's a boundary
    min_chunk_size: usize,
    max_chunk_size: usize,
    boundaries: Vec<usize>,
    pub hashes: Vec<String>,
    byte_index: usize,
}

impl<RH: RollingHasher, H: Hasher> Slicer<RH, H>
{
    pub(crate) fn new(
        rolling_hasher: RH,
        hasher: H,
        boundary_mask: u32,
        min_chunk_size: usize,
        max_chunk_size: usize,
    ) -> Slicer<RH, H> {
        assert!(
            min_chunk_size >= rolling_hasher.get_window_size(),
            "min_chunk_size must be greater than or equal the hasher sliding window size"
        );
        assert!(
            max_chunk_size >= min_chunk_size,
            "max_chunk_size cannot be lower min_chunk_size"
        );
        // hasher.reset();
        Slicer {
            rolling_hasher,
            hasher,
            boundary_mask,
            min_chunk_size,
            max_chunk_size,
            boundaries: vec![],
            hashes: vec![],
            byte_index: 0,
        }
    }

    // pub(crate) fn reset(&mut self) {
    //     self.rolling_hasher.reset();
    //     // self.hasher.reset();
    //     self.boundaries.clear();
    //     self.byte_index = 0;
    //     self.hasher.
    //     self.chunk_buf.clear();
    // }

    pub(crate) fn process(&mut self, buffer: &[u8]) {
        for byte in buffer {
            let rolling_hash = self.rolling_hasher.push(*byte);
            let chunk_size = self.hasher.get_buffer_size();
            if (chunk_size >= self.min_chunk_size
                && (rolling_hash & self.boundary_mask) == 0)
                || chunk_size == self.max_chunk_size
            {
                // compute sha hash
                let hash = self.hasher.finalize();
                println!("{}", hash);
                self.hashes.push(hash);
                self.boundaries.push(self.byte_index);
                self.rolling_hasher.reset();        // TODO: do we need this?
            } 

            self.hasher.push(*byte);
            self.byte_index += 1;
        }
    }
}

#[test]
fn test_slicer() {
    use crate::read_file;
    use super::rolling_hasher::polynomial::*;
    use super::hasher::sha256::*;
    use super::hasher::sha1::*;
    use super::hasher::md5::*;

    let min_chunk_size: usize = 32;
    let max_chunk_size: usize = 8192;
    let rolling_hash_window_size: u32 = 16;
    let rolling_hash_modulus: u32 = 1000000007;
    let rolling_hash_base: u32 = 29791;
    let boundary_mask: u32 = (1 << 6) - 1; // 6 least significant bits set, chunk size is 2^6 bytes on average

    println!("ORIGINAL:");
    let rolling_hasher = PolynomialRollingHasher::new(
        rolling_hash_window_size,
        Some(rolling_hash_modulus), 
        Some(rolling_hash_base)
    );
    let hasher = Sha256Hasher::new(max_chunk_size);
    let mut old_file_slicer = Slicer::new(rolling_hasher, hasher, boundary_mask, min_chunk_size, max_chunk_size);
    read_file("./original.rtf", |bytes, progress| {
        old_file_slicer.process(bytes);
    });
    // for boundary in &old_file_slicer.boundaries {
    //     println!("{}, ", boundary);
    // }
    assert_eq!(old_file_slicer.boundaries.len(), 31);

    println!("MODIFIED:");
    let rolling_hasher = PolynomialRollingHasher::new(
        rolling_hash_window_size,
        Some(rolling_hash_modulus), 
        Some(rolling_hash_base)
    );
    let hasher = Sha256Hasher::new(max_chunk_size);
    let mut new_file_slicer = Slicer::new(rolling_hasher, hasher, boundary_mask, min_chunk_size, max_chunk_size);
    read_file("./modified.rtf", |bytes, progress| {
        new_file_slicer.process(bytes);
    });
    // for boundary in &new_file_slicer.boundaries {
    //     println!("{}, ", boundary);
    // }
    assert_eq!(new_file_slicer.boundaries.len(), 32);
}
