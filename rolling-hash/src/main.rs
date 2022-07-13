use std::env;
use reader::*;
use slicer::rolling_hasher::polynomial::*;
use slicer::hasher::sha256::*;
// use slicer::hasher::sha256::*;
use slicer::slicer::*;
use crate::lcs_nakatsu::*;

mod reader;
mod helper;
mod slicer;
mod lcs;
mod lcs_nakatsu;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        help();
        return
    }

    let old_file_path = &args[1];
    let new_file_path = &args[2];


    /* 

    STEP 1: Read both files and slice them into chunks
    
    This step uses rolling hash algorithm to efficiently perform content-based slicing
    The chunk boundaries depend on the patterns detected in the content and are not prone
    to the shifted boundary issues.
    */

    let boundary_mask: u32 = (1 << 6) - 1; // 6 least significant bits set, chunk size is 2^6 bytes on average

    // let old_rolling_hasher = PolynomialRollingHasher::new(16, Some(1000000007), Some(29791));
    // let old_hasher = Sha256::new();
    // let mut old_file_slicer = Slicer::new(old_rolling_hasher, old_hasher, boundary_mask, 32, 8192);
    // read_file(old_file_path, |bytes, progress| {
    //     old_file_slicer.process(bytes);
    // });

//    rolling_hasher.reset();

    // let new_rolling_hasher = PolynomialRollingHasher::new(16, Some(1000000007), Some(29791));
    // let new_hasher = Sha256::new();
    // let mut new_file_slicer = Slicer::new(new_rolling_hasher, new_hasher, boundary_mask, 32, 8192);
    // read_file(new_file_path, |bytes, progress| {
    //     old_file_slicer.process(bytes);
    // });

    // TODO: is the hasher reset here?
    // let mut new_file_slicer = Slicer::new(hasher, boundary_mask, 32, 8192);
    // read_file(new_file_path, |bytes, progress| {
    //     new_file_slicer.process(bytes);
    // });

    
    /*
    
    STEP 2: Compute the fingerprint for each chunk (hash)

    The streams of hashes will then be compared to determine which chunks need to be
    replaced, inserted and removed.
    We need a proper collision-resistant hash here. This is because we won't perform 
    any strict comparison of the chunks. When hashes match we assume the chunks contain
    the same data and don't need to be updated.
    One may argue that no has is collision-free and we may end up with wrong data after
    applying a patch. While true, one has to note that the collision probability of long
    hashes is very low, even when compared to cosmic bit flips so maybe we shouldn't worry
    and do the best we can
    */
    
    // for chunk in &slicer.chunks {
    //     println!("{}, {}", chunk.upper_byte_index, chunk.simple_hash);
    // }

    /* 
    
    STEP 3: Determine Longest Common Subsequence of the sequences of hashes

    We use the LCS to determine matching chunks from both input sequences. Those
    matching chunks don't need to be sent over the network because the client already
    has them. We will only include the missing chunks (as well as deletions) in our delta
    (patch) file
    */

    // let lcs = lcs_nakatsu(&old_file_slicer.hashes, &new_file_slicer.hashes);
}

fn help() {
    println!("usage:
rolling-hash <old_file> <new_file>
    Finds differences and returns delta.");
}