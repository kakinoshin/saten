
#[derive(Debug, Clone)]
pub enum CompressionType {
    Uncompress,
    Unsupported,
    Deflate,
    Rar5,
    Rar4,
}

#[derive(Debug, Clone)]
pub struct MemberFile {
    pub filepath: String,
    pub filename: String,
    pub offset: u64,
    pub size: u64,
    pub fsize: u64,
    pub ctype: CompressionType,
}

pub trait ArcReader {
    fn new() -> Self;
    fn read_archive(buf : &Vec<u8>, files : &mut Vec<MemberFile>)-> Result<(), Box<dyn std::error::Error>>;
    fn read_data(buf : &Vec<u8>, offset : u64, size : u64) -> Vec<u8>;
}
