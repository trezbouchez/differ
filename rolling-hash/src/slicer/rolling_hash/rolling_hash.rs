pub(crate) trait RollingHash {
    fn push(&mut self, byte: u8);       // pushes new input value
    fn reset(&mut self);                // resets hasher
    fn get_rolling_hash(&self) -> u32;  // hash computed on current window
    fn get_overall_hash(&self) -> u32;  // hash computed since last reset
    fn get_window_size(&self) -> usize;
}
