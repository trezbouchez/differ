use super::hasher::*;
use sha2::{Sha256, Digest};

pub(crate) struct Sha256Hasher {
    buffer: Vec<u8>,
}

impl Hasher for Sha256Hasher {

    #[inline(always)]
    fn push(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    #[inline(always)]
    fn finalize(&mut self) -> String {                       // returns hash
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(&self.buffer);
            hasher.finalize()
        };
        self.buffer.clear();

        format!("{:X}", hash)
    }
}

impl Sha256Hasher {

    #[allow(dead_code)]
    pub(crate) fn new(max_chunk_size: usize) -> Sha256Hasher {
        Sha256Hasher {
            buffer: Vec::with_capacity(max_chunk_size),
        }
    }
}