use super::rolling_hash::rolling_hash::*;

pub(crate) struct Chunk {
    upper_byte_index: usize, // not included in chunk
    simple_hash: u32,        // collission-prone hash
}

pub(crate) struct Slicer<H> {
    hasher: H,
    boundary_mask: u32, // if masked hash bits are all zeros, it's a boundary
    min_chunk_size: usize,
    max_chunk_size: usize,
    chunks: Vec<Chunk>,
    byte_index: usize,
    current_chunk_size: usize,
}

impl<H> Slicer<H>
where
    H: RollingHash,
{
    pub(crate) fn new(
        hasher: H,
        boundary_mask: u32,
        min_chunk_size: usize,
        max_chunk_size: usize,
    ) -> Slicer<H> {
        assert!(
            min_chunk_size >= hasher.get_window_size(),
            "min_chunk_size must be greater than or equal the hasher sliding window size"
        );
        assert!(
            max_chunk_size >= min_chunk_size,
            "max_chunk_size cannot be lower min_chunk_size"
        );
        Slicer {
            hasher,
            boundary_mask,
            chunks: vec![],
            min_chunk_size,
            max_chunk_size,
            byte_index: 0,
            current_chunk_size: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.hasher.reset();
        self.chunks.clear();
        self.byte_index = 0;
        self.current_chunk_size = 0;
    }

    pub(crate) fn process(&mut self, buffer: &[u8]) {
        for byte in buffer {
            self.hasher.push(*byte);
            if (self.current_chunk_size >= self.min_chunk_size
                && (self.hasher.get_rolling_hash() & self.boundary_mask) == 0)
                || self.current_chunk_size == self.max_chunk_size
            {
                self.chunks.push(Chunk {
                    upper_byte_index: self.byte_index,
                    simple_hash: self.hasher.get_overall_hash(),
                });
                self.hasher.reset();
                self.current_chunk_size = 0;
            } else {
                self.current_chunk_size += 1;
            }
            self.byte_index += 1;
        }
    }
}

#[test]
fn test_slicer() {
    use crate::read_file;
    use crate::PolynomialRollingHash;

    let hasher = PolynomialRollingHash::new(16, Some(1000000007), Some(29791));
    let boundary_mask: u32 = (1 << 6) - 1; // 6 least significant bits set, chunk size is 2^6 bytes on average
    let mut slicer = Slicer::new(hasher, boundary_mask, 32, 8192);

    read_file("./original.rtf", |bytes, progress| {
        slicer.process(bytes);
    });
    assert_eq!(slicer.chunks.len(), 31);
    println!("ORIGINAL:");
    for chunk in &slicer.chunks {
        println!("{}, {}", chunk.upper_byte_index, chunk.simple_hash);
    }

    slicer.reset();
    read_file("./modified.rtf", |bytes, progress| {
        slicer.process(bytes);
    });
    assert_eq!(slicer.chunks.len(), 31);
    println!("MODIFIED:");
    for chunk in &slicer.chunks {
        println!("{}, {}", chunk.upper_byte_index, chunk.simple_hash);
    }
}
