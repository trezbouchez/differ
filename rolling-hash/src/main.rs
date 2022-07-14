use std::{
    env,
    io::{stdout, Write},
    // thread::sleep,
    // time::Duration,
};
use reader::*;
use rolling_hasher::polynomial::*;
use hasher::sha256::*;
use lcs::nakatsu::*;
use slicer::*;

mod reader;
mod helper;
mod slicer;
mod rolling_hasher;
mod hasher;
mod lcs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        help();
        return
    }

    let old_file_path = &args[1];
    let new_file_path = &args[2];

    let mut stdout = stdout();


    /* 

    STEP 1: Analyze both files and slice them into fingerprinted chunks
    
    This step uses rolling hash algorithm to efficiently perform content-based slicing
    The chunk boundaries depend on the patterns detected in the content and are not prone
    to the shifted boundary issues.
    The fingerprinting (assigning a collision-resistant digest to each chunk) is performed
    by the hasher instance.

    */

    let min_chunk_size: usize = 32;
    let max_chunk_size: usize = 8192;
    let rolling_hash_window_size: u32 = 16;
    let rolling_hash_modulus: u32 = 1000000007;
    let rolling_hash_base: u32 = 29791;
    let boundary_mask: u32 = (1 << 6) - 1; // 6 least significant bits set, chunk size is 2^6 bytes on average

    // We could get away with sharing the instances of the rolling hasher and the hasher to analyze both files
    // However, at some point we may want to analyze the files concurrently and since these hashers are stateful, 
    // we'd run into problems. Instead, we prefer to create one instance per each slicer.
    let old_file_rolling_hasher = PolynomialRollingHasher::new(
        rolling_hash_window_size,
        Some(rolling_hash_modulus), 
        Some(rolling_hash_base)
    );
    let old_file_hasher = Sha256Hasher::new(max_chunk_size);
    let mut old_file_slicer = Slicer::new(old_file_rolling_hasher, old_file_hasher, boundary_mask, min_chunk_size, max_chunk_size);
    read_file(old_file_path, |bytes, progress| {
        print!("\rProcessing old file {}%", progress);
        stdout.flush().unwrap();
        old_file_slicer.process(bytes);
    });
    old_file_slicer.finalize();
    println!("\rProcessing old file - 100%");

    let new_file_rolling_hasher = PolynomialRollingHasher::new(
        rolling_hash_window_size,
        Some(rolling_hash_modulus), 
        Some(rolling_hash_base)
    );
    let new_file_hasher = Sha256Hasher::new(max_chunk_size);
    let mut new_file_slicer = Slicer::new(new_file_rolling_hasher, new_file_hasher, boundary_mask, min_chunk_size, max_chunk_size);
    read_file(new_file_path, |bytes, progress| {
        print!("\rProcessing new file {}%", progress);
        stdout.flush().unwrap();
        new_file_slicer.process(bytes);
    });
    new_file_slicer.finalize();
    println!("\rProcessing new file - 100%");


    /*
    
    STEP 2: Compute the Longest Common Subsequence of the stream of fingerprints for each
    analyzed file. This will become the basis to generate the patch (delta) file.

    One may argue that no hash is collision-free and we may end up with wrong data after
    applying a patch. While true one has to note that the collision probability of long
    hashes is very low, even when compared to cosmic bit flips so maybe we shouldn't worry
    to much and accept being thrown into a not-ideal world (although close to perfection).

    */

    print!("Computing Longest Common Subsequence...");

    let _ = lcs_nakatsu(&old_file_slicer.hashes, &new_file_slicer.hashes);
    
    println!("\rComputing Longest Common Subsequence - done");


    /*
    
    STEP 3: Create patch (delta) file

    */

    print!("Building delta file...");

    println!("\rBuilding delta file - done");

    // println!("Delta file saved!");

}

fn help() {
    println!("usage:
rolling-hash <old_file> <new_file>
    Finds differences and returns delta.");
}