use rayon::prelude::*;
use std::path::PathBuf;

use crate::{decompress::decompress_file, models::DecompressionResult};

pub fn decompress_files_parallel(
    files: Vec<PathBuf>,
    output_dir: &str,
) -> Vec<DecompressionResult> {
    files
        .into_par_iter()
        .map(|file_path| {
            decompress_file(&file_path, output_dir)
                .unwrap_or_else(|_| {
                    DecompressionResult {
                        filename: file_path
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        compressed_size: 0,
                        decompressed_size: 0,
                        success: false,
                    }
                })
        })
        .collect()
}
