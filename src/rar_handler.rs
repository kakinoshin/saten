use crate::archive_reader::{ArcReader, ArchiveError, ArchiveResult, MemberFile, CompressionType};
use crate::reader_rar4::Rar4Reader;
use crate::reader_rar5::Rar5Reader;
use log::{info, warn, debug};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RarVersion {
    Rar4,
    Rar5,
    Unknown,
}

pub struct RarHandler;

impl RarHandler {
    /// RARファイルのバージョンを判定
    pub fn detect_rar_version(buf: &[u8]) -> RarVersion {
        // RAR5 signature: Rar!\x1a\x07\x01\x00
        if buf.len() >= 8 && buf.starts_with(b"Rar!\x1a\x07\x01\x00") {
            return RarVersion::Rar5;
        }

        // RAR4 signature: Rar!\x1a\x07\x00
        if buf.len() >= 7 && buf.starts_with(b"Rar!\x1a\x07\x00") {
            return RarVersion::Rar4;
        }

        // より詳細な検索
        for window in buf.windows(8) {
            if window == b"Rar!\x1a\x07\x01\x00" {
                return RarVersion::Rar5;
            }
        }

        for window in buf.windows(7) {
            if window == b"Rar!\x1a\x07\x00" {
                return RarVersion::Rar4;
            }
        }

        RarVersion::Unknown
    }

    /// 自動的にRAR形式を判定してファイルリストを読み取り
    pub fn read_archive(buf: &[u8], files: &mut Vec<MemberFile>) -> ArchiveResult<RarVersion> {
        let version = Self::detect_rar_version(buf);
        
        match version {
            RarVersion::Rar4 => {
                info!("Detected RAR4 format");
                Rar4Reader::read_archive(buf, files)?;
                Ok(RarVersion::Rar4)
            }
            RarVersion::Rar5 => {
                info!("Detected RAR5 format");
                Rar5Reader::read_archive(buf, files)?;
                Ok(RarVersion::Rar5)
            }
            RarVersion::Unknown => {
                Err(ArchiveError::UnsupportedFormat)
            }
        }
    }

    /// 圧縮ファイルの展開（バージョン自動判定）
    pub fn extract_file(
        buf: &[u8],
        file: &MemberFile,
        version: RarVersion,
    ) -> ArchiveResult<Vec<u8>> {
        match file.ctype {
            CompressionType::Uncompress => {
                // 無圧縮ファイル（両バージョン共通）
                Self::read_uncompressed_data(buf, file)
            }
            CompressionType::Rar4 => {
                if version != RarVersion::Rar4 {
                    return Err(ArchiveError::CorruptedArchive {
                        message: "RAR4 compression type but not RAR4 format".to_string(),
                    });
                }
                Self::extract_rar4_file(buf, file)
            }
            CompressionType::Rar5 => {
                if version != RarVersion::Rar5 {
                    return Err(ArchiveError::CorruptedArchive {
                        message: "RAR5 compression type but not RAR5 format".to_string(),
                    });
                }
                Self::extract_rar5_file(buf, file)
            }
            _ => {
                Err(ArchiveError::DecompressionError(
                    "Unsupported compression type".to_string()
                ))
            }
        }
    }

    /// 無圧縮データの読み取り
    fn read_uncompressed_data(buf: &[u8], file: &MemberFile) -> ArchiveResult<Vec<u8>> {
        let start = file.offset as usize;
        let end = start + file.size as usize;

        if end > buf.len() {
            return Err(ArchiveError::OutOfBounds {
                offset: file.offset,
                size: file.size,
                buffer_len: buf.len(),
            });
        }

        Ok(buf[start..end].to_vec())
    }

    /// RAR4圧縮ファイルの展開
    fn extract_rar4_file(buf: &[u8], file: &MemberFile) -> ArchiveResult<Vec<u8>> {
        // RAR4の基本的な圧縮方法を使用
        // 実際の実装では、ファイルのメソッド情報が必要
        crate::reader_rar4::decompress_rar4_data(
            buf,
            file.offset,
            file.size,
            file.fsize,
            15, // デフォルトのRAR4メソッド
        )
    }

    /// RAR5圧縮ファイルの展開
    fn extract_rar5_file(buf: &[u8], file: &MemberFile) -> ArchiveResult<Vec<u8>> {
        // RAR5の基本的な圧縮方法を使用
        crate::reader_rar5::decompress_rar5_data(
            buf,
            file.offset,
            file.size,
            file.fsize,
            1, // デフォルトのRAR5メソッド
        )
    }

    /// アーカイブの詳細情報を取得
    pub fn get_archive_info(buf: &[u8]) -> ArchiveResult<ArchiveInfo> {
        let mut files = Vec::new();
        let version = Self::read_archive(buf, &mut files)?;

        let total_files = files.len();
        let total_compressed_size: u64 = files.iter().map(|f| f.size).sum();
        let total_uncompressed_size: u64 = files.iter().map(|f| f.fsize).sum();

        let compression_ratio = if total_uncompressed_size > 0 {
            ((total_uncompressed_size - total_compressed_size) as f64 / total_uncompressed_size as f64) * 100.0
        } else {
            0.0
        };

        let file_types = Self::analyze_file_types(&files);

        Ok(ArchiveInfo {
            version,
            total_files,
            total_compressed_size,
            total_uncompressed_size,
            compression_ratio,
            file_types,
            files,
        })
    }

    /// ファイルタイプの分析
    fn analyze_file_types(files: &[MemberFile]) -> std::collections::HashMap<String, usize> {
        let mut file_types = std::collections::HashMap::new();

        for file in files {
            let extension = file.filename
                .rfind('.')
                .map(|pos| file.filename[pos + 1..].to_lowercase())
                .unwrap_or_else(|| "no extension".to_string());

            *file_types.entry(extension).or_insert(0) += 1;
        }

        file_types
    }

    /// 特定のファイルを名前で検索
    pub fn find_file_by_name<'a>(files: &'a [MemberFile], name: &str) -> Option<&'a MemberFile> {
        files.iter().find(|file| {
            file.filename.eq_ignore_ascii_case(name) || 
            file.filepath.eq_ignore_ascii_case(name)
        })
    }

    /// 特定の拡張子のファイルを検索
    pub fn find_files_by_extension<'a>(files: &'a [MemberFile], extension: &str) -> Vec<&'a MemberFile> {
        let ext_lower = extension.to_lowercase();
        files.iter().filter(|file| {
            file.filename
                .rfind('.')
                .map(|pos| file.filename[pos + 1..].to_lowercase() == ext_lower)
                .unwrap_or(false)
        }).collect()
    }
}

#[derive(Debug)]
pub struct ArchiveInfo {
    pub version: RarVersion,
    pub total_files: usize,
    pub total_compressed_size: u64,
    pub total_uncompressed_size: u64,
    pub compression_ratio: f64,
    pub file_types: std::collections::HashMap<String, usize>,
    pub files: Vec<MemberFile>,
}

impl ArchiveInfo {
    /// 人間が読みやすい形式で情報を表示
    pub fn display(&self) {
        println!("=== RAR Archive Information ===");
        println!("Format: {:?}", self.version);
        println!("Total files: {}", self.total_files);
        println!("Compressed size: {:.2} MB", self.total_compressed_size as f64 / 1024.0 / 1024.0);
        println!("Uncompressed size: {:.2} MB", self.total_uncompressed_size as f64 / 1024.0 / 1024.0);
        println!("Compression ratio: {:.1}%", self.compression_ratio);
        
        println!("\nFile types:");
        let mut sorted_types: Vec<_> = self.file_types.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));
        
        for (ext, count) in sorted_types.iter().take(10) {
            println!("  {}: {} files", ext, count);
        }
        
        if sorted_types.len() > 10 {
            println!("  ... and {} more types", sorted_types.len() - 10);
        }
    }
}

// 使用例とテスト
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rar_version_detection() {
        let rar4_sig = b"Rar!\x1a\x07\x00some data";
        let rar5_sig = b"Rar!\x1a\x07\x01\x00some data";
        let invalid = b"Not a RAR file";

        assert_eq!(RarHandler::detect_rar_version(rar4_sig), RarVersion::Rar4);
        assert_eq!(RarHandler::detect_rar_version(rar5_sig), RarVersion::Rar5);
        assert_eq!(RarHandler::detect_rar_version(invalid), RarVersion::Unknown);
    }

    #[test]
    fn test_file_search() {
        let files = vec![
            MemberFile {
                filepath: "folder/test.txt".to_string(),
                filename: "test.txt".to_string(),
                offset: 0,
                size: 100,
                fsize: 150,
                ctype: CompressionType::Uncompress,
            },
            MemberFile {
                filepath: "image.jpg".to_string(),
                filename: "image.jpg".to_string(),
                offset: 100,
                size: 5000,
                fsize: 5000,
                ctype: CompressionType::Rar5,
            },
        ];

        // 名前での検索
        let found = RarHandler::find_file_by_name(&files, "test.txt");
        assert!(found.is_some());
        assert_eq!(found.unwrap().filename, "test.txt");

        // 拡張子での検索
        let txt_files = RarHandler::find_files_by_extension(&files, "txt");
        assert_eq!(txt_files.len(), 1);

        let jpg_files = RarHandler::find_files_by_extension(&files, "jpg");
        assert_eq!(jpg_files.len(), 1);
    }
}
