
pub(crate) trait RollingHash {
    fn push(&mut self, byte: u8);       // pushes new input value
    fn reset(&mut self);                // resets hasher
    fn hash(&self) -> u32;              // gets current hash value
}
