/*
    This is a simple patcher mainly used for local testing purposes. It takes an old and new file
    paths as well as the patched file path and builds the patched file from old/new using the delta
    array provided (array of segments)
*/

use crate::delta::*;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Result, Seek, SeekFrom, Write},
};

pub(crate) fn patch(
    old_file_path: &str,
    new_file_path: &str,
    patched_file_path: &str,
    segments: Vec<Segment>,
) -> Result<(usize,usize)> {        // returns (old_bytes, new_bytes) - how many bytes were used from old and new 
    let old_file = File::open(old_file_path)?;
    let new_file = File::open(new_file_path)?;
    let mut patched_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(patched_file_path)?;
    let mut old_bytes_used: usize = 0;
    let mut new_bytes_used: usize = 0;
    for segment in segments {
        let (mut source_file, range) = match segment {
            Segment::Old(range) => { 
                old_bytes_used += range.len();
                (&old_file, range)
            },
            Segment::New(range) => {
                new_bytes_used += range.len();
                (&new_file, range)
            },
        };
        // pretty bad way of reading a file, where each chunk requires new heap allocation
        // anyway, good enough for a test
        let mut buffer: Vec<u8> = vec![0; range.len()];
        source_file.seek(SeekFrom::Start(u64::try_from(range.start).unwrap()))?;
        source_file.read_exact(&mut buffer[..])?;
        let bytes_written = patched_file.write(&buffer)?;
        assert_eq!(bytes_written, range.len());
    }
    patched_file.flush()?;

    Ok((old_bytes_used, new_bytes_used))
}
