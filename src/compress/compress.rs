use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
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


pub fn compress_files_parallel(
    files: Vec<PathBuf>,
    output_dir: &str,
    num_threads: usize,
    compression_level: u32,
) -> Vec<CompressionResult> {

    let files = Arc::new(files);
    let output_dir = Arc::new(output_dir.to_string());
    let results = Arc::new(Mutex::new(Vec::new()));

    let chunk_size = (files.len() + num_threads - 1) / num_threads;

    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let files = Arc::clone(&files);
        let output_dir = Arc::clone(&output_dir);
        let results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            let start = thread_id * chunk_size;
            let end = ((thread_id + 1) * chunk_size).min(files.len());

            for file in &files[start..end] {
                println!("Thread {}: Compressing {:?}", thread_id, file.file_name().unwrap());

                match compress_file(file, &output_dir, compression_level) {
                    Ok(result) => results.lock().unwrap().push(result),
                    Err(e) => eprintln!("Error compressing {:?}: {}", file, e),
                }
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Thread panicked");
    }

    Arc::try_unwrap(results)
        .expect("Arc still has multiple owners")
        .into_inner()
        .unwrap()
}

#[cfg(test)]
mod tests{
    use super::*;
    use std::{io::Write};

    #[test]
    fn test_compress_file(){
        let input = "test.input.txt";
        let mut f = File::create(input).unwrap();
        writeln!(f,"hello world welcome to the new world").unwrap();

        let output = "test_output";
        std::fs::create_dir_all(output).unwrap();

        let _result = compress_file(Path::new(input), output, 6).unwrap();

        let expected_ans = format!("{}/test.input.txt.gz", output);
        assert!(Path::new(&expected_ans).exists());

    }
}