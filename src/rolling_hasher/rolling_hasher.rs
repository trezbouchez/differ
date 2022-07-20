/*
    Rolling hasher interface, to be used with Slicer
*/

pub(crate) trait RollingHasher {
    fn push(&mut self, byte: u8) -> u32;        // pushes new input value and returns current hash
    fn get_window_size(&self) -> usize;
}
