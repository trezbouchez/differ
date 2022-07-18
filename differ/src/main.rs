use differ::*;
use patcher::patch;
use reader::*;
use std::{
    env,
    fs::OpenOptions,
    io::Write,
};

mod delta;
mod differ;
mod hasher;
mod helper;
mod lcs;
mod patcher;
mod reader;
mod rolling_hasher;
mod slicer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        help();
        return;
    }

    let old_file_path = &args[1];
    let new_file_path = &args[2];
    let patched_file_path = &args[3];
    let delta_file_path = &args[4];

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

    // save delta
    println!("Saving delta");
    let segments_text = format!("{:?}", segments);
    _ = OpenOptions::new()
        .write(true)
        .create(true)
        .open(delta_file_path).expect("Could not open delta file for writing")
        .write(segments_text.as_bytes());

    // recreate new file by patching the old one
    println!("Patching");
    let (bytes_old, bytes_new) = patch(old_file_path, new_file_path, patched_file_path, segments)
        .expect("Could not apply a patch!");

    println!("Done!");

    let percent_reused: usize = 100 * bytes_old / (bytes_new + bytes_old);
    println!(
        "{} bytes ({}%) have been reused, {} bytes ({}%) have been added.",
        bytes_old,
        percent_reused,
        bytes_new,
        100 - percent_reused
    );
}

fn help() {
    println!("usage:
rolling-hash <old_file> <new_file> <patched_file> <delta_file>
    Creates patched_file identical to new_file by reusing as much of an old file as possible. Will save edits in a delta_file");
}
