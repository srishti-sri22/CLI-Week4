use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use cli_app::compress::compress_files_parallel::compress_files_parallel;
use cli_app::decompress_files::decompress_rayon::decompress_files_parallel;

#[test]
fn test_compress_and_decompress() {
    let input_file = "tests/test_input.txt";
    let mut f = File::create(input_file).unwrap();
    writeln!(f, "hello world to the wonderful wolrd of this world").unwrap();

    let compressed_dir = "tests/compressed";
    let decompressed_dir = "tests/decompressed";
    fs::create_dir_all(compressed_dir).unwrap();
    fs::create_dir_all(decompressed_dir).unwrap();

    let compressed_results = compress_files_parallel(
        vec![Path::new(input_file).to_path_buf()],
        compressed_dir,
        2, 6   
    );
    assert_eq!(compressed_results.len(), 1);

    let compressed_files: Vec<_> = fs::read_dir(compressed_dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    let decompressed_results = decompress_files_parallel(
        compressed_files,
        decompressed_dir
    );

    assert!(!decompressed_results.iter().any(|r| !r.success));

    fs::remove_file(input_file).unwrap();
    fs::remove_dir_all(compressed_dir).unwrap();
    fs::remove_dir_all(decompressed_dir).unwrap();
}
