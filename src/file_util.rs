use std::fs::{File, OpenOptions};
use std::io::prelude::*;

use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec;

use crate::chunk::Block;

///
/// fn main() {
///     save_to_file(b"I'm bill", "bill.txt");
/// }
pub fn save_to_file(data: &[u8], path: &str) {
    //println!("Uncompressed size: {} bytes", data.len());
    let data = compress_to_vec(data, 8);
    //println!("Compressed size: {} bytes", data.as_slice().len());

    let f = OpenOptions::new().write(true).create(true).open(path);

    match f {
        Ok(mut f) => {
            f.write_all(data.as_slice()).expect("Unable to write data");
        }
        Err(e) => {
            println!("Error saving chunk: {e}");
        }
    }
}

///
/// fn main() {
///     match read_from_file("bill.txt") {
///         Ok(s) => println!("{}", s),
///         Err(e) => println!("Failed to read file: {:?}", e),
///     }
///  }
pub fn read_chunk_data_from_file(path: &str) -> Option<Box<ndarray::Array3<Block>>> {
    if let Ok(mut f) = File::open(path) {
        let mut data = Vec::new();
        f.read_to_end(&mut data).expect("Unable to read data");

        let decompressed = decompress_to_vec(data.as_slice());

        match decompressed {
            Ok(decompressed) => {
                let serialized = bincode::deserialize(decompressed.as_slice());
                match serialized {
                    Err(e) => {
                        println!("Error reading chunk: {e}");
                        None
                    }
                    Ok(chunk_data) => Some(Box::new(chunk_data)),
                }
            }
            Err(_) => None,
        }
    } else {
        None
    }
}
