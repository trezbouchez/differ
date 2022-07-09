use super::rolling_hash::rolling_hash::*;

pub(crate) struct Slicer<H> {
    hasher: H,
    boundary_mask: u32,         // if masked hash bits are all zeros, it's a boundary
    boundaries: Vec<usize>,
    index: usize,
}

impl<H> Slicer<H>
where
    H: RollingHash,
{
    pub(crate) fn new(hasher: H, boundary_mask: u32) -> Slicer<H> {
        Slicer {
            hasher,
            boundary_mask,
            boundaries: vec![0],
            index: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        // self.hasher.reset();
        self.boundaries.clear();
    }

    pub(crate) fn process(&mut self, buffer: &[u8]) {
        for byte in buffer {
            self.hasher.push(*byte);
            if self.hasher.hash() & self.boundary_mask == 0 {
                // there's no need to reset if we ignore boundaries
                // detected in the first window_size bytes
//                self.hasher.reset();  
                self.boundaries.push(self.index);
            }
            self.index = self.index + 1;
        }
    }
}

#[test]
fn test_slicer() {
    use crate::PolynomialSlidingWindowHash;
    use crate::read_file;

    let hasher = PolynomialSlidingWindowHash::new(64, Some(1000000007), Some(29791));
    let boundary_mask: u32 = (1 << 6) - 1;      // 6 least significant bits set
    let mut slicer = Slicer::new(hasher, boundary_mask);

    read_file("./original.rtf", |bytes, progress|{
        slicer.process(bytes);
    });

    // TODO: not a robust test, we should at least test avg slice size for various masks 
    assert!(slicer.boundaries.len() == 44);
}