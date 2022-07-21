use super::hasher::hasher::*;
use super::rolling_hasher::rolling_hasher::*;

/*

Slicer partitions the stream into content-based chunks and fingerprints them.

Chunk size depends on:
- boundary_mask, which determines average chunk size (for random input)
- min_chunk_size, max_chunk_size, which set the allowed chunk length range

The content-based boundary detection requires RollingHasher trait-implementing
instance, injected as the 'rolling_hasher' argument to 'new'

Once a chunk boundaries are known, the proper collision-resistant digest will
be assigned to it. The computation is performed by a Hasher trait-implementing
instance passed as a 'hasher' argument to 'new'.

The Slicer instance is being fed with bytes of the analyzed stream to its 'process'
associated function.
When the stream ends the 'finalize' must be called to correctly terminate the last chunk.

The result of the Slicer processing are:
- boundaries, which holds start indices of each chunk (and the length of the stream as last)
- hashes, containing collision-resistant hashes of each chunk

Slicer cannot be reset. It is mean for analyzing a single stream. Create new instance if
another stream needs to be analyzed.

*/

pub(crate) struct Chunk {
    pub hash: Vec<u8>,
    pub end: usize,
}

pub(crate) struct Slicer<RH: RollingHasher, H: Hasher> {
    rolling_hasher: RH,
    hasher: H,
    boundary_mask: u32, // if masked hash bits are all zeros, it's a boundary
    min_chunk_size: usize,
    max_chunk_size: usize,
    current_chunk_size: usize,
    current_chunk_start: usize,
    chunks: Vec<Chunk>,
}

impl<RH: RollingHasher, H: Hasher> Slicer<RH, H> {
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
        Slicer {
            rolling_hasher,
            hasher,
            boundary_mask,
            min_chunk_size,
            max_chunk_size,
            current_chunk_size: 0,
            current_chunk_start: 0,
            chunks: vec![],
        }
    }

    pub(crate) fn process(&mut self, buffer: &[u8]) {
        for byte in buffer {
            let rolling_hash = self.rolling_hasher.push(*byte); // compute rolling hash
            if (self.current_chunk_size >= self.min_chunk_size
                && (rolling_hash & self.boundary_mask) == 0)
                || self.current_chunk_size == self.max_chunk_size
            {
                self.add_chunk();
            }
            self.hasher.push(*byte);
            self.current_chunk_size += 1;
        }
    }

    pub(crate) fn finalize(&mut self) -> &Vec<Chunk> {
        self.add_chunk();
        &self.chunks
    }

    fn add_chunk(&mut self) {
        let hash = self.hasher.finalize();
        let chunk_end = self.current_chunk_start + self.current_chunk_size;
        let chunk = Chunk {
            hash,
            end: chunk_end,
        };
        self.chunks.push(chunk);
        self.current_chunk_start = chunk_end;
        self.current_chunk_size = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hasher::sha256::*;
    use crate::rolling_hasher::polynomial::*;
    use crate::read_file;

    #[test]
    #[should_panic(
        expected = r#"min_chunk_size must be greater than or equal the hasher sliding window size"#
    )]
    fn test_slicer_min_chunk_size_wrong() {
        // To avoid the need to reset rolling hash on each boundary detection we ensure it keeps
        // running for at least window_size before the next chunk can be detected (so that all irrelevant
        // values in the buffer get overwritten). For this to work, slicer's min_chunk_size must be
        // greater than or equal the rolling hash window_size
        let min_chunk_size: usize = 32;
        let max_chunk_size: usize = 8192;
        let rolling_hash_window_size: u32 = 64;
        let rolling_hash_modulus: u32 = 1000000007;
        let rolling_hash_base: u32 = 29791;
        let boundary_mask: u32 = (1 << 6) - 1; // 6 least significant bits set, chunk size is 2^6 bytes on average

        let rolling_hasher = PolynomialRollingHasher::new(
            rolling_hash_window_size,
            Some(rolling_hash_modulus),
            Some(rolling_hash_base),
        );
        let hasher = Sha256Hasher::new(max_chunk_size);
        _ = Slicer::new(
            rolling_hasher,
            hasher,
            boundary_mask,
            min_chunk_size,
            max_chunk_size,
        );
    }

    #[test]
    fn test_slicer() {
        let min_chunk_size: usize = 2048;
        let max_chunk_size: usize = 8129;
        let rolling_hash_window_size: u32 = 32;
        let rolling_hash_modulus: u32 = 1000000007;
        let rolling_hash_base: u32 = 29791;
        let boundary_mask: u32 = (1 << 12) - 1; // avg chunk size is 2^12 = 4096 bytes on average

        let rolling_hasher = PolynomialRollingHasher::new(
            rolling_hash_window_size,
            Some(rolling_hash_modulus),
            Some(rolling_hash_base),
        );
        let hasher = Sha256Hasher::new(max_chunk_size);
        let mut old_file_slicer = Slicer::new(
            rolling_hasher,
            hasher,
            boundary_mask,
            min_chunk_size,
            max_chunk_size,
        );
        read_file("./example/monkey_before.tiff", |bytes, _| {
            old_file_slicer.process(bytes);
        });
        old_file_slicer.finalize();

        // got 69 chunks for a file size of ~353KB, avg chunk size is 5115 bytes
        assert_eq!(old_file_slicer.chunks.len(), 69);
    }
}
