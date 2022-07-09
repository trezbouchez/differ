use std::env;
use reader::*;
use slicer::rolling_hash::polynomial::*;
use slicer::slicer::*;

mod reader;
mod helper;
mod slicer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        help();
        return
    }

    let old_file_path = &args[1];
    let new_file_path = &args[2];
}

fn help() {
    println!("usage:
rolling-hash <old_file> <new_file>
    Finds differences and returns delta.");
}