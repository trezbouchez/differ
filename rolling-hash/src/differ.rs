use crate::hasher::hasher::*;
use crate::rolling_hasher::rolling_hasher::*;
use crate::slicer::*;
use crate::lcs::nakatsu::*; 
use crate::delta::*;

pub(crate) struct Differ<RH: RollingHasher, H: Hasher> {
    slicer_old: Slicer<RH, H>,
    slicer_new: Slicer<RH, H>,
    is_finalized: bool,
}

impl<RH: RollingHasher, H: Hasher> Differ<RH, H> {
    pub(crate) fn new(slicer_old: Slicer<RH, H>, slicer_new: Slicer<RH, H>) -> Differ<RH, H> {
        Differ {
            slicer_old,
            slicer_new,
            is_finalized: false,
        }
    }

    // Pass buffers as you read both files. Once both files have been processed, calling
    // finalize will compute the delta file
    pub(crate) fn process_old(&mut self, buffer: &[u8]) {
        assert!(
            !self.is_finalized,
            "Alrady finalized, cannot accept more input."
        );
        self.slicer_old.process(buffer);
    }

    pub(crate) fn process_new(&mut self, buffer: &[u8]) {
        assert!(
            !self.is_finalized,
            "Alrady finalized, cannot accept more input."
        );
        self.slicer_new.process(buffer);
    }

    pub(crate) fn finalize(&mut self) {
        assert!(!self.is_finalized, "Alrady finalized!");
        self.is_finalized = true;

        self.slicer_old.finalize();
        self.slicer_new.finalize();

        // let lcs = 
        // Compute Longest Common Subseqence
        // let old_hashes: [String] = self.slicer_old.chunks.iter().map(|chunk| chunk.hash).collect();
        // let new_hashes = self.slicer_new.chunks.iter().map(|chunk| chunk.hash);

        // let lcs = lcs_nakatsu(&old_hashes.collect(), &new_hashes);

        // // Generate delta
        // delta(&self.slicer_old.hashes, &self.slicer_new.hashes, &lcs[..]);
    }
}
