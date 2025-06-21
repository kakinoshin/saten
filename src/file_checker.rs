use crate::archive_reader::{ArchiveError, ArchiveResult};

// ファイル形式のシグネチャ定数
const RAR5_SIGNATURE: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00];
const RAR4_SIGNATURE: &[u8] = &[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];
const ZIP_SIGNATURE: &[u8] = &[0x50, 0x4B, 0x03, 0x04];

#[derive(Debug, PartialEq, Clone)]
pub enum FileType {
    Zip,
    Rar5,
    Rar4,
    Unsupported
}

pub fn check_file_type(buf: &[u8]) -> ArchiveResult<FileType> {
    if buf.is_empty() {
        return Err(ArchiveError::CorruptedArchive {
            message: "空のファイルです".to_string(),
        });
    }

    if check_signature(buf, RAR5_SIGNATURE) {
        Ok(FileType::Rar5)
    } else if check_signature(buf, RAR4_SIGNATURE) {
        Ok(FileType::Rar4)
    } else if check_signature(buf, ZIP_SIGNATURE) {
        Ok(FileType::Zip)
    } else {
        Ok(FileType::Unsupported)
    }
}

/// 指定されたシグネチャがバッファ内に存在するかチェック
fn check_signature(data: &[u8], signature: &[u8]) -> bool {
    if data.len() < signature.len() {
        return false;
    }

    // ファイルの先頭のシグネチャをチェック
    &data[0..signature.len()] == signature
}
