use thiserror::Error;
use std::string::FromUtf8Error;

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("ファイル読み取りエラー: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("サポートされていないアーカイブ形式")]
    UnsupportedFormat,
    
    #[error("破損したアーカイブファイル: {message}")]
    CorruptedArchive { message: String },
    
    #[error("ファイルサイズが不正です: expected {expected}, found {found}")]
    InvalidFileSize { expected: usize, found: usize },
    
    #[error("ヘッダー解析エラー: {0}")]
    HeaderParseError(String),
    
    #[error("文字列変換エラー: {0}")]
    StringConversionError(#[from] FromUtf8Error),
    
    #[error("圧縮解除エラー: {0}")]
    DecompressionError(String),
    
    #[error("範囲外アクセス: offset {offset}, size {size}, buffer_len {buffer_len}")]
    OutOfBounds { offset: u64, size: u64, buffer_len: usize },
    
    #[error("画像処理エラー: {0}")]
    ImageError(#[from] image::ImageError),
}

pub type ArchiveResult<T> = Result<T, ArchiveError>;

#[derive(Debug, Clone)]
pub enum CompressionType {
    Uncompress,
    Unsupported,
    Deflate,
    Deflate64,
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
    fn read_archive(buf: &[u8], files: &mut Vec<MemberFile>) -> ArchiveResult<()>;
    fn read_data(buf: &[u8], offset: u64, size: u64) -> ArchiveResult<Vec<u8>>;
}
