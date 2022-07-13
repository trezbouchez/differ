/*
This serves as a wrapper around various cryptographic hash crates. 
It exposes uniform interface and provides data buffering.
Structs implementing this trait are reusable - after finalize
is called a new hash is computed on the buffered data and the buffer 
gets cleared.
*/

pub(crate) trait Hasher {
    fn push(&mut self, byte: u8);
    fn finalize(&mut self) -> String;                        // returns hash and resets
    fn get_buffer_size(&self) -> usize;
}
