use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use log::{info, warn, error, debug};

use crate::archive_reader::{ArcReader, ArchiveError, ArchiveResult, MemberFile, CompressionType};
use crate::reader_rar5::Rar5Reader;
use crate::reader_rar4::Rar4Reader;
use crate::reader_zip::ZipReader;
use crate::file_checker::{FileType, check_file_type};
use crate::sort_filename::sort_filename;
use crate::compress_deflate;

pub struct ArchiveManager;

impl ArchiveManager {
    pub fn new() -> Self {
        Self
    }

    /// アーカイブファイルを読み込む
    pub fn load_archive_file(path: &PathBuf) -> ArchiveResult<Vec<u8>> {
        let file_path = path.to_str()
            .ok_or_else(|| ArchiveError::CorruptedArchive {
                message: "無効なファイルパスです".to_string(),
            })?;

        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        info!("ファイルを読み込みました: {} ({} bytes)", file_path, buffer.len());
        Ok(buffer)
    }

    /// アーカイブを解析してファイルリストを作成
    pub fn process_archive(buffer: &[u8]) -> ArchiveResult<Vec<MemberFile>> {
        let file_type = check_file_type(buffer)?;
        let mut files = Vec::new();
        
        match file_type {
            FileType::Rar5 => {
                info!("ファイル形式: RAR5");
                Rar5Reader::read_archive(buffer, &mut files)?;
            },
            FileType::Rar4 => {
                info!("ファイル形式: RAR4");
                Rar4Reader::read_archive(buffer, &mut files)?;
            },
            FileType::Zip => {
                info!("ファイル形式: ZIP");
                ZipReader::read_archive(buffer, &mut files)?;
            },
            FileType::Unsupported => {
                return Err(ArchiveError::UnsupportedFormat);
            }
        }
        
        sort_filename(&mut files);
        info!("アーカイブの処理が完了: {} 個のファイルを検出", files.len());
        
        Ok(files)
    }

    /// ファイルデータを解凍
    pub fn decompress_file_data(
        buffer: &[u8], 
        file: &MemberFile
    ) -> ArchiveResult<Vec<u8>> {
        match file.ctype {
            CompressionType::Uncompress => {
                Self::read_uncompressed_data(buffer, file.offset, file.size)
            },
            CompressionType::Deflate | CompressionType::Deflate64 => {
                compress_deflate::uncomp_deflate(buffer, file.offset, file.size)
            },
            CompressionType::Rar5 | CompressionType::Rar4 => {
                Err(ArchiveError::DecompressionError(
                    "RAR圧縮はまだサポートされていません".to_string()
                ))
            },
            CompressionType::Unsupported => {
                Err(ArchiveError::DecompressionError(
                    "サポートされていない圧縮形式です".to_string()
                ))
            }
        }
    }

    /// 非圧縮データを読み取り
    fn read_uncompressed_data(
        buffer: &[u8], 
        offset: u64, 
        size: u64
    ) -> ArchiveResult<Vec<u8>> {
        let start = offset as usize;
        let end = start + size as usize;
        
        if end > buffer.len() {
            return Err(ArchiveError::OutOfBounds {
                offset,
                size,
                buffer_len: buffer.len(),
            });
        }
        
        Ok(buffer[start..end].to_owned())
    }

    /// ファイル情報を検証
    pub fn validate_file_info(file: &MemberFile) -> bool {
        !file.filepath.is_empty() && file.size > 0
    }

    /// サポートされている圧縮形式かチェック
    pub fn is_supported_compression(compression_type: &CompressionType) -> bool {
        matches!(compression_type, 
            CompressionType::Uncompress | 
            CompressionType::Deflate | 
            CompressionType::Deflate64
        )
    }

    /// アーカイブ内のファイル数を取得
    pub fn get_file_count(files: &[MemberFile]) -> usize {
        files.len()
    }

    /// 指定インデックスのファイルが有効かチェック
    pub fn is_valid_index(files: &[MemberFile], index: usize) -> bool {
        index < files.len()
    }
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self::new()
    }
}
