use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    
    #[arg(short, long)]
    pub input: String,

    
    #[arg(short, long)]
    pub output: String,

    
    #[arg(short, long, default_value_t = 4)]
    pub threads: usize,

    
    #[arg(short, long, default_value_t = 6)]
    pub level: u32,
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub filename: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: f64,
}

#[derive(Debug)]
pub struct DecompressionResult {
    pub filename: String,
    pub compressed_size: u64,
    pub decompressed_size: u64,
    pub success: bool,
}
