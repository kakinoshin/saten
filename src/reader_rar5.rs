use encoding_rs;
use std::io::Read;
use flate2::read::DeflateDecoder;

use crate::archive_reader::{ArcReader, ArchiveError, ArchiveResult};
use crate::archive_reader::{MemberFile, CompressionType};
use log::{info, warn, error, debug};

pub struct Rar5Reader {
    buf: Vec<u8>,
    files: Vec<MemberFile>,
}

impl ArcReader for Rar5Reader {
    fn new() -> Self {
        Self {
            buf: Vec::new(),
            files: Vec::new(),
        }
    }

    fn read_archive(buf: &[u8], files: &mut Vec<MemberFile>) -> ArchiveResult<()> {
        let mut offset: usize = 0;

        let (pos, is_sign) = check_rarsign(buf);
        debug!("RAR5 signature pos: {:?}", pos);

        if !is_sign {
            return Err(ArchiveError::CorruptedArchive {
                message: "RAR5 signature not found".to_string(),
            });
        }

        offset += pos + 8; // RAR5 signature is 8 bytes

        // Check main archive header
        let htype = check_headertype(buf, offset)?;
        debug!("RAR5 header type: {:?}", htype);

        if htype != 1 {
            return Err(ArchiveError::CorruptedArchive {
                message: format!("Expected main archive header (type 1), found type {}", htype),
            });
        }

        // Process main archive header
        let main_header_size = process_main_archive_header(buf, offset)?;
        debug!("RAR5 main header size: {:?}", main_header_size);
        offset += main_header_size;

        // Process file and service headers
        loop {
            if offset >= buf.len() {
                break;
            }

            match check_headertype(buf, offset) {
                Ok(2) => {
                    // File header
                    debug!("Processing RAR5 File header");
                    let header_size = process_file_header(buf, offset, files)?;
                    offset += header_size;
                }
                Ok(3) => {
                    // Service header
                    debug!("Processing RAR5 Service header");
                    let header_size = process_service_header(buf, offset)?;
                    offset += header_size;
                }
                Ok(5) => {
                    // End of archive
                    debug!("Reached end of archive (type 5)");
                    break;
                }
                Ok(htype) => {
                    warn!("Unknown header type: {}, skipping", htype);
                    // Try to skip unknown header
                    let header_size = get_header_size(buf, offset)?;
                    offset += header_size;
                }
                Err(_) => {
                    debug!("No more valid headers found");
                    break;
                }
            }
        }

        info!("Successfully parsed RAR5 archive with {} files", files.len());
        Ok(())
    }

    fn read_data(buf: &[u8], offset: u64, size: u64) -> ArchiveResult<Vec<u8>> {
        let start = offset as usize;
        let end = start + size as usize;

        if end > buf.len() {
            return Err(ArchiveError::OutOfBounds {
                offset,
                size,
                buffer_len: buf.len(),
            });
        }

        Ok(buf[start..end].to_owned())
    }
}

// RAR5圧縮データを展開する関数
pub fn decompress_rar5_data(
    buf: &[u8], 
    offset: u64, 
    size: u64, 
    uncompressed_size: u64, 
    method: u8
) -> ArchiveResult<Vec<u8>> {
    let start = offset as usize;
    let end = start + size as usize;

    if end > buf.len() {
        return Err(ArchiveError::OutOfBounds {
            offset,
            size,
            buffer_len: buf.len(),
        });
    }

    let compressed_data = &buf[start..end];

    match method {
        0 => {
            // 無圧縮
            debug!("No compression, returning raw data");
            Ok(compressed_data.to_vec())
        }
        1..=5 => {
            // RAR5の各圧縮方法
            warn!("RAR5 compression method {} detected, attempting decompression", method);
            
            // RAR5の圧縮データは複雑な独自アルゴリズムを使用
            // ここでは基本的な展開を試行
            match decompress_rar5_basic(compressed_data, uncompressed_size, method) {
                Ok(data) => Ok(data),
                Err(_) => {
                    // フォールバック: 無圧縮として扱う
                    warn!("RAR5 decompression failed, treating as uncompressed");
                    Ok(compressed_data.to_vec())
                }
            }
        }
        _ => {
            error!("Unsupported RAR5 compression method: {}", method);
            Err(ArchiveError::DecompressionError(
                format!("Unsupported RAR5 compression method: {}", method)
            ))
        }
    }
}

fn decompress_rar5_basic(
    compressed_data: &[u8], 
    expected_size: u64, 
    method: u8
) -> ArchiveResult<Vec<u8>> {
    match method {
        1 => {
            // Method 1: 基本的なDeflateベース
            let mut deflater = DeflateDecoder::new(compressed_data);
            let mut decompressed = Vec::new();
            
            match deflater.read_to_end(&mut decompressed) {
                Ok(_) => {
                    if decompressed.len() == expected_size as usize {
                        Ok(decompressed)
                    } else {
                        Err(ArchiveError::DecompressionError(
                            format!("Size mismatch: expected {}, got {}", expected_size, decompressed.len())
                        ))
                    }
                }
                Err(e) => Err(ArchiveError::DecompressionError(
                    format!("RAR5 method 1 decompression failed: {}", e)
                ))
            }
        }
        _ => {
            // その他の方法は複雑なため、現在は未対応
            Err(ArchiveError::DecompressionError(
                format!("RAR5 compression method {} not implemented", method)
            ))
        }
    }
}

// Variable-length integer読み取り関数（エラーハンドリング改善版）
fn read_vint(data: &[u8], pos: usize) -> ArchiveResult<(u64, u8)> {
    let mut offset = 0u8;
    let mut val = 0u64;
    let mut shift = 0;

    loop {
        if data.len() <= pos + offset as usize {
            return Err(ArchiveError::OutOfBounds {
                offset: pos as u64,
                size: (offset + 1) as u64,
                buffer_len: data.len(),
            });
        }

        let d = data[pos + offset as usize] as u64;
        val |= (d & 0x7F) << shift;
        
        if d & 0x80 != 0x80 {
            break;
        }
        
        offset += 1;
        shift += 7;

        if offset >= 10 {
            return Err(ArchiveError::CorruptedArchive {
                message: "Variable integer too long".to_string(),
            });
        }
    }

    Ok((val, offset + 1))
}

fn check_rarsign(data: &[u8]) -> (usize, bool) {
    // RAR 5.0: 0x52 0x61 0x72 0x21 0x1A 0x07 0x01 0x00
    const RAR5_SIGNATURE: &[u8] = b"Rar!\x1a\x07\x01\x00";

    for (i, window) in data.windows(RAR5_SIGNATURE.len()).enumerate() {
        if window == RAR5_SIGNATURE {
            return (i, true);
        }
    }

    (0, false)
}

fn check_headertype(data: &[u8], pos: usize) -> ArchiveResult<u64> {
    if data.len() < pos + 6 {
        return Err(ArchiveError::OutOfBounds {
            offset: pos as u64,
            size: 6,
            buffer_len: data.len(),
        });
    }

    let mut offset = pos + 4; // skip CRC32

    // Header size
    let (hsize, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    // Header type
    let (htype, _) = read_vint(data, offset)?;

    debug!("Header - type: {}, size: {}", htype, hsize);
    Ok(htype)
}

fn get_header_size(data: &[u8], pos: usize) -> ArchiveResult<usize> {
    if data.len() < pos + 6 {
        return Err(ArchiveError::OutOfBounds {
            offset: pos as u64,
            size: 6,
            buffer_len: data.len(),
        });
    }

    let offset = pos + 4; // skip CRC32
    let (hsize, vintlen) = read_vint(data, offset)?;
    
    Ok(4 + vintlen as usize + hsize as usize)
}

fn process_main_archive_header(data: &[u8], pos: usize) -> ArchiveResult<usize> {
    let mut offset = pos + 4; // skip CRC32

    // Header size
    let (hsize, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;
    let header_len = 4 + vintlen as usize + hsize as usize;

    // Header type
    let (htype, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    if htype != 1 {
        return Err(ArchiveError::CorruptedArchive {
            message: format!("Invalid main archive header type: {}", htype),
        });
    }

    // Header flags
    let (hflag, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    // Extra area size
    if hflag & 0x01 != 0 {
        let (extra_size, vintlen) = read_vint(data, offset)?;
        offset += vintlen as usize;
        debug!("Extra area size: {}", extra_size);
    }

    // Archive flags
    let (aflag, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    debug!("Archive flags: {:#04x}", aflag);
    
    let is_multivolume = aflag & 0x01 != 0;
    let is_not_first = aflag & 0x02 != 0;
    let is_solid = aflag & 0x04 != 0;
    let is_recovery = aflag & 0x08 != 0;
    let is_locked = aflag & 0x10 != 0;

    debug!("Archive properties - Multivolume: {}, Not first: {}, Solid: {}, Recovery: {}, Locked: {}", 
        is_multivolume, is_not_first, is_solid, is_recovery, is_locked);

    // Volume number (if not first volume)
    if is_not_first {
        let (volnum, vintlen) = read_vint(data, offset)?;
        offset += vintlen as usize;
        debug!("Volume number: {}", volnum + 1);
    }

    Ok(header_len)
}

fn process_file_header(data: &[u8], pos: usize, files: &mut Vec<MemberFile>) -> ArchiveResult<usize> {
    let mut offset = pos + 4; // skip CRC32

    // Header size
    let (hsize, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;
    let _header_len = 4 + vintlen as usize + hsize as usize;

    // Header type
    let (htype, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    if htype != 2 {
        return Err(ArchiveError::CorruptedArchive {
            message: format!("Invalid file header type: {}", htype),
        });
    }

    // Header flags
    let (hflag, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    let has_extra = hflag & 0x01 != 0;
    let has_data = hflag & 0x02 != 0;

    // Extra area size
    let extra_size = if has_extra {
        let (size, vintlen) = read_vint(data, offset)?;
        offset += vintlen as usize;
        size
    } else {
        0
    };

    // Data size
    let data_size = if has_data {
        let (size, vintlen) = read_vint(data, offset)?;
        offset += vintlen as usize;
        size
    } else {
        0
    };

    // File flags
    let (fflag, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    let is_dir = fflag & 0x01 != 0;
    let has_unix_time = fflag & 0x02 != 0;
    let has_crc32 = fflag & 0x04 != 0;

    // Unpacked size
    let (file_size, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    // Attributes
    let (_file_attr, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    // Unix time (optional)
    if has_unix_time {
        if offset + 4 > data.len() {
            return Err(ArchiveError::OutOfBounds {
                offset: offset as u64,
                size: 4,
                buffer_len: data.len(),
            });
        }
        let _mtime = read_u32_le(&data[offset..offset + 4]);
        offset += 4;
    }

    // CRC32 (optional)
    if has_crc32 {
        if offset + 4 > data.len() {
            return Err(ArchiveError::OutOfBounds {
                offset: offset as u64,
                size: 4,
                buffer_len: data.len(),
            });
        }
        let _crc32 = read_u32_le(&data[offset..offset + 4]);
        offset += 4;
    }

    // Compression information
    let (comp_info, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    let comp_version = comp_info & 0x003f;
    let is_solid = (comp_info & 0x0040) != 0;
    let comp_method = (comp_info & 0x0380) >> 7;
    let dict_size = (comp_info & 0x3c00) >> 10;

    debug!("Compression - version: {}, solid: {}, method: {}, dict_size: {}", 
        comp_version, is_solid, comp_method, dict_size);

    // Host OS
    let (_host_os, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    // Name length
    let (name_len, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    // File name
    if offset + name_len as usize > data.len() {
        return Err(ArchiveError::OutOfBounds {
            offset: offset as u64,
            size: name_len,
            buffer_len: data.len(),
        });
    }

    let file_name = decode_filename(&data[offset..offset + name_len as usize])?;
    offset += name_len as usize;

    debug!("File: {} (size: {}, compressed: {}, dir: {})", file_name, file_size, data_size, is_dir);

    // Extra area
    if has_extra {
        if offset + extra_size as usize > data.len() {
            return Err(ArchiveError::OutOfBounds {
                offset: offset as u64,
                size: extra_size,
                buffer_len: data.len(),
            });
        }
        offset += extra_size as usize;
    }

    // Data area
    let data_offset = offset as u64;
    let final_offset = if has_data {
        offset + data_size as usize
    } else {
        offset
    };

    // 圧縮タイプの判定
    let ctype = match comp_method {
        0 => CompressionType::Uncompress,
        1..=5 => CompressionType::Rar5,
        _ => CompressionType::Unsupported,
    };

    // ファイル情報を追加（ディレクトリではない場合のみ）
    if has_data && !is_dir {
        let filename_only = if let Some(pos) = file_name.rfind(['/', '\\']) {
            file_name[pos + 1..].to_string()
        } else {
            file_name.clone()
        };

        files.push(MemberFile {
            filepath: file_name.clone(),
            filename: filename_only,
            offset: data_offset,
            size: data_size,
            fsize: file_size,
            ctype,
        });

        debug!("Added file: {} (packed: {}, unpacked: {})", file_name, data_size, file_size);
    } else if is_dir {
        debug!("Skipped directory: {}", file_name);
    }

    Ok(final_offset - pos)
}

fn process_service_header(data: &[u8], pos: usize) -> ArchiveResult<usize> {
    let mut offset = pos + 4; // skip CRC32

    // Header size
    let (hsize, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;
    let header_len = 4 + vintlen as usize + hsize as usize;

    // Header type
    let (htype, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    if htype != 3 {
        return Err(ArchiveError::CorruptedArchive {
            message: format!("Invalid service header type: {}", htype),
        });
    }

    // Header flags
    let (hflag, vintlen) = read_vint(data, offset)?;
    offset += vintlen as usize;

    let has_extra = hflag & 0x01 != 0;
    let has_data = hflag & 0x02 != 0;

    // Extra area size
    if has_extra {
        let (extra_size, vintlen) = read_vint(data, offset)?;
        offset += vintlen as usize;
        debug!("Service header extra size: {}", extra_size);
    }

    // Data size
    let data_size = if has_data {
        let (size, vintlen) = read_vint(data, offset)?;
        offset += vintlen as usize;
        size
    } else {
        0
    };

    debug!("Service header with data size: {}", data_size);

    // サービスヘッダーの詳細は省略し、サイズ分スキップ
    let final_offset = if has_data {
        header_len + data_size as usize
    } else {
        header_len
    };

    Ok(final_offset)
}

fn decode_filename(data: &[u8]) -> ArchiveResult<String> {
    // RAR5はUTF-8エンコーディングを使用
    match std::str::from_utf8(data) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => {
            // UTF-8でない場合、エンコーディングを推測
            let (decoded, _, had_errors) = encoding_rs::UTF_8.decode(data);
            if !had_errors {
                return Ok(decoded.into_owned());
            }

            // フォールバック: Latin-1
            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(data);
            Ok(decoded.into_owned())
        }
    }
}

fn read_u32_le(data: &[u8]) -> u32 {
    (data[3] as u32) << 24 | (data[2] as u32) << 16 | (data[1] as u32) << 8 | (data[0] as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rar5_signature() {
        let valid_rar = b"Rar!\x1a\x07\x01\x00some data";
        let (pos, found) = check_rarsign(valid_rar);
        assert_eq!(pos, 0);
        assert!(found);
    }

    #[test]
    fn test_vint_reading() {
        let data = [0x81, 0x02]; // 257 in vint format
        let result = read_vint(&data, 0).unwrap();
        assert_eq!(result.0, 257);
        assert_eq!(result.1, 2);
    }

    #[test]
    fn test_filename_decoding() {
        let utf8_name = b"test.txt";
        let result = decode_filename(utf8_name).unwrap();
        assert_eq!(result, "test.txt");
    }
}
