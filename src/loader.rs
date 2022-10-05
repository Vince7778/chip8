
use std::fs;

pub fn get_file_bytes(file_path: &String) -> Result<Vec<u8>, std::io::Error> {
    fs::read(file_path)
}
