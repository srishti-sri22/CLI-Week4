use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path};

use crate::models;
use models::{CompressionResult};

pub fn compress_file(
    input_path: &Path,
    output_dir: &str,
    compression_level: u32,
) -> io::Result<CompressionResult> {

    let buffer = fs::read(input_path).expect("Incorrect buffer read");
    let original_size = buffer.len()  as u64;

    let filename = input_path
        .file_name()
        .unwrap()
        .to_string_lossy();

    let output_path = Path::new(output_dir).join(format!("{}.gz", filename));

    let output_file = File::create(&output_path)?;
    let mut encoder = GzEncoder::new(output_file, Compression::new(compression_level));
    encoder.write_all(&buffer)?;
    encoder.finish().expect("The encoder stream did not finish");

    let compressed_size = fs::metadata(&output_path)?.len();

    let compression_ratio = if original_size > 0 && compressed_size < original_size {
        ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };

    Ok(CompressionResult {
        filename: filename.into_owned(),
        original_size,
        compressed_size,
        compression_ratio,
    })
}




// #[cfg(test)]
// mod tests{
//     use super::*;
//     use std::{io::Write};

//     #[test]
//     fn test_compress_file(){
//         let input = "test.input.txt";
//         let mut f = File::create(input).unwrap();
//         writeln!(f,"hello world welcome to the new world").unwrap();

//         let output = "test_output";
//         std::fs::create_dir_all(output).unwrap();

//         let _result = compress_file(Path::new(input), output, 6).unwrap();

//         let expected_ans = format!("{}/test.input.txt.gz", output);
//         assert!(Path::new(&expected_ans).exists());

//     }
// }