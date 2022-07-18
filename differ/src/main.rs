use std::env;
use reader::*;
use differ::*;
use patcher::patch;

mod reader;
mod helper;
mod slicer;
mod rolling_hasher;
mod hasher;
mod lcs;
mod differ;
mod delta;
mod patcher;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        help();
        return
    }

    let old_file_path = &args[1];
    let new_file_path = &args[2];
    let patched_file_path = &args[3];

    let min_chunk_size: usize = 2048;
    let max_chunk_size: usize = 8192;
    let rolling_hash_window_size: u32 = 16;
    let boundary_mask: u32 = (1 << 12) - 1; // average chunk size is 2^12 = 4096 bytes

    let mut differ = Differ::new(
        Some(rolling_hash_window_size),
        Some(min_chunk_size),
        Some(max_chunk_size),
        Some(boundary_mask),
    );

    // slice the old file and compute hashes (they could be analyzed concurrently, too)
    println!("Processing old file");
    read_file(old_file_path, |bytes, _| {
        differ.process_old(bytes);
    });

    // slice the new file and compute hashes
    println!("Processing new file");
    read_file(new_file_path, |bytes, _| {
        differ.process_new(bytes);
    });

    // compute longest common subsequence and determine delta
    println!("Computing delta");
    let segments = differ.finalize();

    // recreate new file by patching the old one
    println!("Patching");
    let (bytes_old, bytes_new) = patch(old_file_path, new_file_path, patched_file_path, segments).expect("Could not apply a patch!");

    println!("Done!");

    let percent_reused: usize = 100 * bytes_old / (bytes_new + bytes_old);
    println!("{} bytes ({}%) have been reused, {} bytes ({}%) have been added.", bytes_old, percent_reused, bytes_new, 100 - percent_reused);
}

fn help() {
    println!("usage:
rolling-hash <old_file> <new_file> <patched_file>
    Creates patched_file identical to new_file by reusing as much of an old file as possible.");
}