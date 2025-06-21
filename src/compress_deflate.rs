use std::io::Read;
use flate2::read::DeflateDecoder;
use crate::archive_reader::{ArchiveError, ArchiveResult};
use log::{info, error};

pub fn uncomp_deflate(buf: &[u8], offset: u64, size: u64) -> ArchiveResult<Vec<u8>> {
    // 範囲チェック
    let start = offset as usize;
    let end = start + size as usize;
    
    if end > buf.len() {
        return Err(ArchiveError::OutOfBounds {
            offset,
            size,
            buffer_len: buf.len(),
        });
    }

    log::info!("Deflate圧縮を解除中: offset={}, size={}", offset, size);
    
    let src = &buf[start..end];
    let mut deflater = DeflateDecoder::new(src);
    let mut data = Vec::new();
    
    deflater.read_to_end(&mut data)
        .map_err(|e| {
            log::error!("Deflate解除エラー: {}", e);
            ArchiveError::DecompressionError(format!("Deflate解除に失敗: {}", e))
        })?;

    Ok(data)
}
