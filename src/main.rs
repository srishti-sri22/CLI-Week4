use clap::Parser;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Parser, Debug)]
#[command(name = "File Compressor")]
#[command(version = "1.0")]
#[command(about = "Compress multiple files in parallel")]
struct Args {
    
    #[arg(short, long)]
    input: String,

    
    #[arg(short, long)]
    output: String,

    
    #[arg(short, long, default_value_t = 4)]
    threads: usize,

    
    #[arg(short, long, default_value_t = 6)]
    level: u32,
}

#[derive(Debug, Clone)]
struct CompressionResult {
    filename: String,
    original_size: u64,
    compressed_size: u64,
    compression_ratio: f64,
}

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

    
    if let Err(e) = fs::create_dir_all(&args.output) {
        eprintln!("Error creating output directory: {}", e);
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

    
    let results = compress_files_parallel(files, &args.output, args.threads, args.level);

    
    println!("\n=== Compression Results ===\n");
    let mut total_original = 0u64;
    let mut total_compressed = 0u64;

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

    println!("=== Summary ===");
    println!("Files compressed: {}", results.len());
    println!("Total original size: {} bytes", total_original);
    println!("Total compressed size: {} bytes", total_compressed);
    println!("Overall compression: {:.2}%", overall_ratio);
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
            // Skip already compressed files
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

fn compress_file(
    input_path: &Path,
    output_dir: &str,
    compression_level: u32,
) -> io::Result<CompressionResult> {
    // Read input file
    let mut input_file = File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;
    let original_size = buffer.len() as u64;

    // Create output file path
    let filename = input_path.file_name().unwrap().to_string_lossy();
    let output_filename = format!("{}.gz", filename);
    let output_path = Path::new(output_dir).join(&output_filename);

    // Compress and write
    let output_file = File::create(&output_path)?;
    let mut encoder = GzEncoder::new(output_file, Compression::new(compression_level));
    encoder.write_all(&buffer)?;
    encoder.finish()?;

    // Get compressed size
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