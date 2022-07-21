use super::hasher::*;
use sha1::{Sha1, Digest};

/* 
WARNING: 
This file uses SHA1 hashing algorithm which is not cryptographically safe anymore.
Still, it's ok to use it for file comparison purposes
*/

pub(crate) struct Sha1Hasher {
    buffer: Vec<u8>,
}

impl Hasher for Sha1Hasher {

    #[inline(always)]
    fn push(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    #[inline(always)]
    fn finalize(&mut self) -> Vec<u8> {                       // returns hash
        let hash = {
            let mut hasher = Sha1::new();
            hasher.update(&self.buffer);
            hasher.finalize().to_vec()
        };
        
        self.buffer.clear();

        hash
    }
}

impl Sha1Hasher {

    #[allow(dead_code)]
    pub(crate) fn new(max_chunk_size: usize) -> Sha1Hasher {
        Sha1Hasher {
            buffer: Vec::with_capacity(max_chunk_size),
        }
    }
}