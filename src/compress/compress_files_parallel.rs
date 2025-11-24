use std::{thread, path::PathBuf, sync::{Arc, Mutex}};

use crate::{compress::compress::compress_file, models::CompressionResult};

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