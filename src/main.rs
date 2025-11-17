use clap::Parser;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
mod models;
use models::{Args, CompressionResult, DecompressionResult};


fn main() {
    let args = Args::parse();

    if args.level > 9 {
        eprintln!("Error: Compression level must be between 0 and 9");
        std::process::exit(1);
    }

    println!("Starting file compression...");
    println!("Input directory: {}", args.input);
    println!("Output directory: {}", args.output);
    println!("Threads: {}", args.threads);
    println!("Compression level: {}\n", args.level);

    let compressed_dir = format!("{}/compressed_gz", args.output);
    let decompressed_dir = format!("{}/decompressed_files", args.output);
    
    if let Err(e) = fs::create_dir_all(&compressed_dir) {
        eprintln!("Error creating compressed directory: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = fs::create_dir_all(&decompressed_dir) {
        eprintln!("Error creating decompressed directory: {}", e);
        std::process::exit(1);
    }

    
    let files = match collect_files(&args.input) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error reading input directory: {}", e);
            std::process::exit(1);
        }
    };

    if files.is_empty() {
        println!("No files found to compress");
        return;
    }

    println!("Found {} files to compress\n", files.len());

    
    let results = compress_files_parallel(files, &compressed_dir, args.threads, args.level);

    
    println!("\n=== Compression Results ===\n");
    let mut total_original = 0;
    let mut total_compressed = 0;

    for result in &results {
        println!("File: {}", result.filename);
        println!("  Original size: {} bytes", result.original_size);
        println!("  Compressed size: {} bytes", result.compressed_size);
        println!("  Compression ratio: {:.2}%\n", result.compression_ratio);

        total_original += result.original_size;
        total_compressed += result.compressed_size;
    }

    let overall_ratio = if total_original > 0 {
        ((total_original - total_compressed) as f64 / total_original as f64) * 100.0
    } else {
        0.0
    };

    println!("Summary");
    println!("Files compressed: {}", results.len());
    println!("Total original size: {} bytes", total_original);
    println!("Total compressed size: {} bytes", total_compressed);
    println!("Overall compression: {:.2}%", overall_ratio);
    println!("Compressed files location: {}", compressed_dir);

    println!("\n\n Starting Decompression \n");

    let compressed_files = match collect_compressed_files(&compressed_dir) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("Error reading compressed files: {}", e);
            std::process::exit(1);
        }
    };

    if compressed_files.is_empty() {
        println!("No compressed files found to decompress");
        return;
    }

    println!("Found {} files to decompress\n", compressed_files.len());

    let decompress_results = decompress_files_parallel(
        compressed_files,
        &decompressed_dir,
        args.threads,
    );

    println!("\n=== Decompression Results ===\n");
    for result in &decompress_results {
        println!("File: {}", result.filename);
        println!("  Compressed size: {} bytes", result.compressed_size);
        println!("  Decompressed size: {} bytes", result.decompressed_size);
        println!("  Status: {}\n", if result.success { "Success" } else { "Failed" });
    }

    println!("Summary");
    println!("Files decompressed: {}", decompress_results.iter().filter(|r| r.success).count());
    println!("Decompressed files location: {}", decompressed_dir);
    
}

fn collect_files(dir: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let path = PathBuf::from(dir);

    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory '{}' not found", dir),
        ));
    }

    if path.is_file() {
        files.push(path);
        return Ok(files);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "gz" {
                    continue;
                }
            }
            files.push(path);
        }
    }

    Ok(files)
}

fn collect_compressed_files(dir: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let path = PathBuf::from(dir);

    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory '{}' not found", dir),
        ));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "gz" {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

fn compress_files_parallel(
    files: Vec<PathBuf>,
    output_dir: &str,
    num_threads: usize,
    compression_level: u32,
) -> Vec<CompressionResult> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let files = Arc::new(files);
    let output_dir = Arc::new(output_dir.to_string());

    let chunk_size = (files.len() + num_threads - 1) / num_threads;
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let files = Arc::clone(&files);
        let results = Arc::clone(&results);
        let output_dir = Arc::clone(&output_dir);

        let handle = thread::spawn(move || {
            let start = thread_id * chunk_size;
            let end = ((thread_id + 1) * chunk_size).min(files.len());

            for i in start..end {
                let file_path = &files[i];
                println!("Thread {}: Compressing {:?}", thread_id, file_path.file_name().unwrap());

                match compress_file(file_path, &output_dir, compression_level) {
                    Ok(result) => {
                        let mut results = results.lock().unwrap();
                        results.push(result);
                    }
                    Err(e) => {
                        eprintln!("Error compressing {:?}: {}", file_path, e);
                    }
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    Arc::try_unwrap(results)
        .expect("Arc still has multiple owners")
        .into_inner()
        .unwrap()
}

fn decompress_files_parallel(
    files: Vec<PathBuf>,
    output_dir: &str,
    num_threads: usize,
) -> Vec<DecompressionResult> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let files = Arc::new(files);
    let output_dir = Arc::new(output_dir.to_string());

    let chunk_size = (files.len() + num_threads - 1) / num_threads;
    let mut handles = vec![];

    for thread_id in 0..num_threads {
        let files = Arc::clone(&files);
        let results = Arc::clone(&results);
        let output_dir = Arc::clone(&output_dir);

        let handle = thread::spawn(move || {
            let start = thread_id * chunk_size;
            let end = ((thread_id + 1) * chunk_size).min(files.len());

            for i in start..end {
                let file_path = &files[i];
                println!("Thread {}: Decompressing {:?}", thread_id, file_path.file_name().unwrap());

                match decompress_file(file_path, &output_dir) {
                    Ok(result) => {
                        let mut results = results.lock().unwrap();
                        results.push(result);
                    }
                    Err(e) => {
                        eprintln!("Error decompressing {:?}: {}", file_path, e);
                        let mut results = results.lock().unwrap();
                        results.push(DecompressionResult {
                            filename: file_path.file_name().unwrap().to_string_lossy().to_string(),
                            compressed_size: 0,
                            decompressed_size: 0,
                            success: false,
                        });
                    }
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    Arc::try_unwrap(results)
        .expect("Arc still has multiple owners")
        .into_inner()
        .unwrap()
}

fn compress_file(
    input_path: &Path,
    output_dir: &str,
    compression_level: u32,
) -> io::Result<CompressionResult> {
    let mut input_file = File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    let original_size = buffer.len() as u64;

    let filename = input_path.file_name().unwrap().to_string_lossy();
    let output_filename = format!("{}.gz", filename);
    let output_path = Path::new(output_dir).join(&output_filename);

    let output_file = File::create(&output_path)?;
    let mut encoder = GzEncoder::new(output_file, Compression::new(compression_level));
    encoder.write_all(&buffer)?;
    encoder.finish()?;

    let compressed_size = fs::metadata(&output_path)?.len();

    let compression_ratio = if original_size > 0 {
        ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };

    Ok(CompressionResult {
        filename: filename.to_string(),
        original_size,
        compressed_size,
        compression_ratio,
    })
}

fn decompress_file(
    input_path: &Path,
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
    output_file.write_all(&buffer)?;

    Ok(DecompressionResult {
        filename: filename.to_string(),
        compressed_size,
        decompressed_size,
        success: true,
    })
}