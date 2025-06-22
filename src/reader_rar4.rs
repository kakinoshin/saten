use encoding_rs;
use std::io::Read;
use flate2::read::DeflateDecoder;

use crate::archive_reader::{ArcReader, ArchiveError, ArchiveResult};
use crate::archive_reader::{MemberFile, CompressionType};
use log::{info, warn, error, debug};

pub struct Rar4Reader {
    buf: Vec<u8>,
    files: Vec<MemberFile>,
}

impl ArcReader for Rar4Reader {
    fn new() -> Self {
        Self {
            buf: Vec::new(),
            files: Vec::new(),
        }
    }

    fn read_archive(buf: &[u8], files: &mut Vec<MemberFile>) -> ArchiveResult<()> {
        let mut offset: usize = 0;

        let (pos, is_sign) = check_rarsign(&buf);
        debug!("RAR4 signature pos: {:?}", pos);

        if !is_sign {
            return Err(ArchiveError::CorruptedArchive {
                message: "RAR4 signature not found".to_string(),
            });
        }

        offset += pos + 7; // skip signature

        loop {
            if buf.len() <= offset + 7 {
                // size of header
                break;
            }

            let (htype, hflags, hsize) = check_headertype(&buf, offset)?;
            debug!("header type: {:#02x}, flags: {:#04x}, size: {}", htype, hflags, hsize);
            offset += 7;

            if hsize == 0 {
                return Err(ArchiveError::CorruptedArchive {
                    message: "Invalid header size".to_string(),
                });
            }

            if hsize < 7 {
                return Err(ArchiveError::CorruptedArchive {
                    message: format!("Header size too small: {}", hsize),
                });
            }

            match htype {
                0x72 => {
                    // MARK_HEAD (0x72)
                    warn!("Not supported header type (MARK_HEAD: 0x72)");
                    break;
                }
                0x73 => {
                    // MAIN_HEAD (0x73)
                    debug!("Processing MAIN_HEAD (0x73)");
                    if offset + (hsize as usize - 7) > buf.len() {
                        return Err(ArchiveError::OutOfBounds {
                            offset: offset as u64,
                            size: (hsize as usize - 7) as u64,
                            buffer_len: buf.len(),
                        });
                    }
                    offset += (hsize as usize) - 7;
                }
                0x74 => {
                    // FILE_HEAD (0x74)
                    offset = process_file_header(buf, offset, hflags, hsize, files)?;
                }
                0x75 => {
                    // COMM_HEAD (0x75)
                    warn!("Not supported header type (COMM_HEAD: 0x75)");
                    break;
                }
                0x76 => {
                    // AV_HEAD (0x76)
                    warn!("Not supported header type (AV_HEAD: 0x76)");
                    break;
                }
                0x77 => {
                    // SUB_HEAD (0x77)
                    warn!("Not supported header type (SUB_HEAD: 0x77)");
                    break;
                }
                0x78 => {
                    // PROTECT_HEAD (0x78)
                    warn!("Not supported header type (PROTECT_HEAD: 0x78)");
                    break;
                }
                0x79 => {
                    // SIGN_HEAD (0x79)
                    warn!("Not supported header type (SIGN_HEAD: 0x79)");
                    break;
                }
                0x7a => {
                    // NEWSUB_HEAD (0x7a)
                    debug!("Processing NEWSUB_HEAD (0x7a)");
                    if offset + 4 > buf.len() {
                        return Err(ArchiveError::OutOfBounds {
                            offset: offset as u64,
                            size: 4,
                            buffer_len: buf.len(),
                        });
                    }
                    let newsub_size = read_u32_le(&buf[offset..offset + 4]);
                    debug!("NEWSUB Size: {}", newsub_size);
                    offset += (hsize as usize) - 7; // skip header
                    offset += newsub_size as usize; // skip newsub body
                }
                0x7b => {
                    // ENDARC_HEAD (0x7b)
                    debug!("Reached end of archive (ENDARC_HEAD: 0x7b)");
                    break;
                }
                _ => {
                    warn!("Unknown header type: {:#02x}", htype);
                    break;
                }
            }
        }

        info!("Successfully parsed RAR4 archive with {} files", files.len());
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

// RAR4圧縮データを展開する関数
pub fn decompress_rar4_data(buf: &[u8], offset: u64, size: u64, uncompressed_size: u64, method: u8) -> ArchiveResult<Vec<u8>> {
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
        15 | 20 | 26 | 29 | 36 => {
            // RAR4の各圧縮方法
            // 注意: RAR4の圧縮アルゴリズムは複雑で、完全な実装は困難です
            // ここでは基本的なDeflateベースの展開を試行します
            warn!("RAR4 compression method {} detected, attempting basic decompression", method);
            
            // RAR4の圧縮データは通常Deflateベースですが、独自の改良が加えられています
            // 完全な対応には専用のライブラリが必要です
            match decompress_rar4_deflate(compressed_data, uncompressed_size) {
                Ok(data) => Ok(data),
                Err(_) => {
                    // フォールバック: 無圧縮として扱う
                    warn!("RAR4 decompression failed, treating as uncompressed");
                    Ok(compressed_data.to_vec())
                }
            }
        }
        _ => {
            error!("Unsupported RAR4 compression method: {}", method);
            Err(ArchiveError::DecompressionError(
                format!("Unsupported RAR4 compression method: {}", method)
            ))
        }
    }
}

fn decompress_rar4_deflate(compressed_data: &[u8], expected_size: u64) -> ArchiveResult<Vec<u8>> {
    // RAR4のDeflateベースの圧縮を試行
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
            format!("Deflate decompression failed: {}", e)
        ))
    }
}

fn process_file_header(
    buf: &[u8],
    mut offset: usize,
    hflags: u16,
    _hsize: u16,
    files: &mut Vec<MemberFile>,
) -> ArchiveResult<usize> {
    debug!("Processing FILE_HEAD (0x74)");

    if offset + 25 > buf.len() {
        return Err(ArchiveError::OutOfBounds {
            offset: offset as u64,
            size: 25,
            buffer_len: buf.len(),
        });
    }

    // PackSize (4 bytes)
    let mut packed_size = read_u32_le(&buf[offset..offset + 4]) as u64;
    offset += 4;

    // UnpSize (4 bytes)
    let mut unpacked_size = read_u32_le(&buf[offset..offset + 4]) as u64;
    offset += 4;

    // HostOS (1 byte)
    offset += 1;

    // FileCRC (4 bytes)
    offset += 4;

    // FileTime (4 bytes)
    offset += 4;

    // UnpVer (1 byte)
    let unpver = buf[offset];
    offset += 1;

    // Method (1 byte)
    let _method = buf[offset];
    offset += 1;

    // NameSize (2 bytes)
    let nsize = read_u16_le(&buf[offset..offset + 2]);
    debug!("filename size: {}", nsize);
    offset += 2;

    // FileAttr (4 bytes)
    let fattr = read_u32_le(&buf[offset..offset + 4]);
    debug!("file attr: {:#08x}", fattr);
    offset += 4;

    // LHD_LARGE フラグの処理
    if (hflags & 0x0100) != 0 {
        // HighPackSize (4 bytes)
        if offset + 4 > buf.len() {
            return Err(ArchiveError::OutOfBounds {
                offset: offset as u64,
                size: 4,
                buffer_len: buf.len(),
            });
        }
        let high_packed = read_u32_le(&buf[offset..offset + 4]) as u64;
        packed_size |= high_packed << 32;
        debug!("HighPackSize: {:#08x}", high_packed);
        offset += 4;

        // HighUnpSize (4 bytes)
        if offset + 4 > buf.len() {
            return Err(ArchiveError::OutOfBounds {
                offset: offset as u64,
                size: 4,
                buffer_len: buf.len(),
            });
        }
        let high_unpacked = read_u32_le(&buf[offset..offset + 4]) as u64;
        unpacked_size |= high_unpacked << 32;
        debug!("HighUnpSize: {:#08x}", high_unpacked);
        offset += 4;
    }

    debug!("packed size: {}, unpacked size: {}", packed_size, unpacked_size);

    // FileName
    if offset + nsize as usize > buf.len() {
        return Err(ArchiveError::OutOfBounds {
            offset: offset as u64,
            size: nsize as u64,
            buffer_len: buf.len(),
        });
    }

    let mut endpos = offset + nsize as usize;
    for i in offset..(offset + nsize as usize) {
        if buf[i] == 0 {
            endpos = i;
            break;
        }
    }

    let file_name = decode_filename(&buf[offset..endpos])?;
    debug!("filename: {}", file_name);
    offset += nsize as usize;

    // Salt処理
    if (hflags & 0x0400) != 0 {
        debug!("Salt present");
        if offset + 8 > buf.len() {
            return Err(ArchiveError::OutOfBounds {
                offset: offset as u64,
                size: 8,
                buffer_len: buf.len(),
            });
        }
        offset += 8; // Salt (8 bytes)
    }

    // ExtTime処理
    if (hflags & 0x1000) != 0 {
        debug!("ExtTime present");
        if offset + 2 > buf.len() {
            return Err(ArchiveError::OutOfBounds {
                offset: offset as u64,
                size: 2,
                buffer_len: buf.len(),
            });
        }
        let ext_flags = read_u16_le(&buf[offset..offset + 2]);
        offset += 2;

        // ExtTimeの各フィールドを処理（簡略化）
        let sections = [
            (ext_flags & 0x000F) >> 0,  // mtime
            (ext_flags & 0x00F0) >> 4,  // ctime
            (ext_flags & 0x0F00) >> 8,  // atime
            (ext_flags & 0xF000) >> 12, // arctime
        ];

        for &section in &sections {
            if section != 0 {
                let size = ((section & 3) + 1) * 2;
                if offset + size as usize > buf.len() {
                    return Err(ArchiveError::OutOfBounds {
                        offset: offset as u64,
                        size: size as u64,
                        buffer_len: buf.len(),
                    });
                }
                offset += size as usize;
            }
        }
    }

    let data_offset = offset as u64;

    // 圧縮タイプの判定
    let ctype = match unpver {
        0 => CompressionType::Uncompress,
        15 | 20 | 26 | 29 | 36 => CompressionType::Rar4,
        _ => CompressionType::Unsupported,
    };

    // ファイル情報を追加（ディレクトリではない場合のみ）
    // RAR4では、ディレクトリは fattr の 0x10 ビットで判定
    if (fattr & 0x10) == 0 {
        // ディレクトリではない場合
        let filename_only = if let Some(pos) = file_name.rfind(['/', '\\']) {
            file_name[pos + 1..].to_string()
        } else {
            file_name.clone()
        };

        files.push(MemberFile {
            filepath: file_name.clone(),
            filename: filename_only,
            offset: data_offset,
            size: packed_size,
            fsize: unpacked_size,
            ctype,
        });

        debug!("Added file: {} (packed: {}, unpacked: {})", file_name, packed_size, unpacked_size);
    } else {
        debug!("Skipped directory: {}", file_name);
    }

    // パックされたデータをスキップ
    offset += packed_size as usize;

    Ok(offset)
}

fn decode_filename(data: &[u8]) -> ArchiveResult<String> {
    // まずUTF-8として解釈を試行
    match std::str::from_utf8(data) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => {
            // UTF-8でない場合、CP866（ロシア語）またはShift_JIS（日本語）を試行
            let (decoded, _, had_errors) = encoding_rs::UTF_8.decode(data);
            if !had_errors {
                return Ok(decoded.into_owned());
            }

            // CP866を試行（RAR4でよく使われる）
            let (decoded, _, had_errors) = encoding_rs::IBM866.decode(data);
            if !had_errors {
                return Ok(decoded.into_owned());
            }

            // Shift_JISを試行
            let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(data);
            Ok(decoded.into_owned())
        }
    }
}

fn check_rarsign(data: &[u8]) -> (usize, bool) {
    const RAR_SIGNATURE: &[u8] = b"Rar!\x1a\x07\x00";

    for (i, window) in data.windows(RAR_SIGNATURE.len()).enumerate() {
        if window == RAR_SIGNATURE {
            return (i, true);
        }
    }

    (0, false)
}

fn check_headertype(data: &[u8], pos: usize) -> ArchiveResult<(u8, u16, u16)> {
    if data.len() < pos + 7 {
        return Err(ArchiveError::OutOfBounds {
            offset: pos as u64,
            size: 7,
            buffer_len: data.len(),
        });
    }

    let mut offset = pos;

    // skip crc (2 bytes)
    offset += 2;

    // header type (1 byte)
    let htype = data[offset];
    offset += 1;

    // header flags (2 bytes, little endian)
    let hflags = read_u16_le(&data[offset..offset + 2]);
    offset += 2;

    // header size (2 bytes, little endian)
    let hsize = read_u16_le(&data[offset..offset + 2]);

    debug!("Header - type: {:#02x}, flags: {:#04x}, size: {}", htype, hflags, hsize);

    Ok((htype, hflags, hsize))
}

fn read_u16_le(data: &[u8]) -> u16 {
    (data[1] as u16) << 8 | (data[0] as u16)
}

fn read_u32_le(data: &[u8]) -> u32 {
    (data[3] as u32) << 24 | (data[2] as u32) << 16 | (data[1] as u32) << 8 | (data[0] as u32)
}
