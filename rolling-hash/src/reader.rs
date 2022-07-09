use std::fs::File;
use std::io::{BufReader, BufRead};

pub const FILE_READER_BUF_SIZE: usize = 16;

pub(crate) fn read_file<F>(path: &str, mut on_read: F) where F: FnMut(&[u8], u64) {

    let file = File::open(path).expect("Could not open file");
    let file_size: usize = file.metadata().expect("Could not read file metadata").len().try_into().unwrap();

    let mut reader = BufReader::with_capacity(FILE_READER_BUF_SIZE, file);
    
    let mut processed_so_far: usize = 0;
    loop {
        let buffer = reader.fill_buf().expect("File read failed");
        let bytes_read: usize = buffer.len().try_into().unwrap();
        if bytes_read == 0 {
            break;
        }
        let progress: u64 = (100 * processed_so_far / file_size).try_into().unwrap();

        on_read(buffer, progress);

        processed_so_far = processed_so_far + bytes_read;
        let length = buffer.len();
        reader.consume(length);
    }
}
