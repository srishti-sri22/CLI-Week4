use std::fs;
use clap::Parser;
mod models;
use models::{Args};
mod compress;


mod decompress_files;
use decompress_files::decompress_rayon::decompress_files_parallel;

mod utils;
use utils::collect_compressed_files::collect_compressed_files;
use utils::collect_files::collect_files;

use crate::compress::compress_files_parallel::compress_files_parallel;

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

    
    println!(" Compression Results");
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
    );

    println!("Decompression Results ");
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


