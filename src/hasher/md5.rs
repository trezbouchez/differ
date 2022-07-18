use super::hasher::*;
use md5;

/* 
WARNING: 
This file uses SHA1 hashing algorithm which is not cryptographically safe anymore.
Still, it's ok to use it for file comparison purposes
*/

pub(crate) struct Md5Hasher {
    buffer: Vec<u8>,
}

impl Hasher for Md5Hasher {

    fn push(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    fn finalize(&mut self) -> String {                       // returns hash
        let hash = md5::compute(&self.buffer);
        
        self.buffer.clear();

        format!("{:X}", hash)
    }
}

impl Md5Hasher {

    #[allow(dead_code)]
    pub(crate) fn new(max_chunk_size: usize) -> Md5Hasher {
        Md5Hasher {
            buffer: Vec::with_capacity(max_chunk_size),
        }
    }
}