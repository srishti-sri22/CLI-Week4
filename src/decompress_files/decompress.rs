use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
// use std::sync::{Arc, Mutex};
// use std::thread;

use crate::models::DecompressionResult;

pub fn decompress_file(
    input_path: &PathBuf,
    output_dir: &str,
) -> io::Result<DecompressionResult> {
    let input_file = File::open(input_path)?;
    let compressed_size = input_file.metadata()?.len();

    let mut decoder = GzDecoder::new(input_file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;

    let decompressed_size = buffer.len() as u64;

    let filename = input_path.file_name().unwrap().to_string_lossy();
    let output_filename = filename.trim_end_matches(".gz");
    let output_path = Path::new(output_dir).join(output_filename);

    let mut output_file = File::create(&output_path)?;
    output_file.write_all(&buffer).expect("Failed to write in the output file");

    Ok(DecompressionResult {
        filename: filename.to_string(),
        compressed_size,
        decompressed_size,
        success: true,
    })
}



// pub fn decompress_files_parallel(
//     files: Vec<PathBuf>,
//     output_dir: &str,
//     num_threads: usize,
// ) -> Vec<DecompressionResult> {
//     let results = Arc::new(Mutex::new(Vec::new()));
//     let files = Arc::new(files);
//     let output_dir = Arc::new(output_dir.to_string());

//     let chunk_size = (files.len() + num_threads - 1) / num_threads;
//     let mut handles = vec![];

//     for thread_id in 0..num_threads {
//         let files = Arc::clone(&files);
//         let results = Arc::clone(&results);
//         let output_dir = Arc::clone(&output_dir);

//         let handle = thread::spawn(move || {
//             let start = thread_id * chunk_size;
//             let end = ((thread_id + 1) * chunk_size).min(files.len());

//             for i in start..end {
//                 let file_path = &files[i];

//                 let result = match decompress_file(file_path, &output_dir) {
//                     Ok(r) => r,
//                     Err(_) => DecompressionResult {
//                         filename: file_path.file_name().unwrap().to_string_lossy().into(),
//                         compressed_size: 0,
//                         decompressed_size: 0,
//                         success: false,
//                     },
//                 };

//                 results.lock().unwrap().push(result);
//             }
//         });

//         handles.push(handle);
//     }

//     for handle in handles {
//         handle.join().unwrap();
//     }

//     Arc::try_unwrap(results).unwrap().into_inner().unwrap()
// } 
