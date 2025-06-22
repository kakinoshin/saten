use std::io::Read;
use flate2::read::DeflateDecoder;

use crate::archive_reader::{ArcReader, ArchiveError, ArchiveResult};
use crate::archive_reader::{MemberFile, CompressionType};
use log::{info, warn, error, debug};

pub struct ZipReader {
    buf: Vec<u8>,
    files: Vec<MemberFile>,
}

impl ArcReader for ZipReader {
    fn new() -> Self {
        Self {
            buf: Vec::new(),
            files: Vec::new(),
        }
    }

    fn read_archive(buf: &[u8], files: &mut Vec<MemberFile>) -> ArchiveResult<()> {
        let mut offset : usize = 0;

        // local file header signature     4 bytes  (0x04034b50)
        let (pos, is_sign) = check_zipsign(&buf)?;
        log::info!("ZIPシグネチャ位置: {}", pos);

        if is_sign {
            offset = pos;
            loop {
                if buf.len() <= offset + 30 {    // size of header
                    break;
                }
                log::debug!("ZIPブロック開始位置: {}", offset);

                // local file header signature     4 bytes  (0x04034b50)
                if buf[offset] == 0x50 &&
                   buf[offset+1] == 0x4B && 
                   buf[offset+2] == 0x03 && 
                   buf[offset+3] == 0x04 {
                    log::debug!("ZIPシグネチャ位置: {}", offset);
                } else {
                    log::warn!("シグネチャが見つかりません");
                    break;
                }
                offset += 4;

                // version needed to extract       2 bytes
                let _ver = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // general purpose bit flag        2 bytes
                let _gpflag = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // compression method              2 bytes
                let comp = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // last mod file time              2 bytes
                let _file_time = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // last mod file date              2 bytes
                let _file_date = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // crc-32                          4 bytes
                let _crc32 = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                offset += 4;
                // compressed size                 4 bytes
                let csize = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                log::debug!("圧縮サイズ: {}", csize);
                offset += 4;
                // uncompressed size               4 bytes
                let ucsize = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                log::debug!("非圧縮サイズ: {}", ucsize);
                offset += 4;
                // file name length                2 bytes
                let fname_size = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // extra field length              2 bytes               
                let ex_length = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // file name (variable size)
                if offset + fname_size as usize > buf.len() {
                    return Err(ArchiveError::CorruptedArchive {
                        message: "ファイル名の範囲が不正です".to_string(),
                    });
                }
                let file_name = std::str::from_utf8(&buf[offset..(offset+fname_size as usize)])
                    .map_err(|_| ArchiveError::CorruptedArchive {
                        message: "ファイル名の文字列変換に失敗しました".to_string(),
                    })?;
                log::info!("ファイル名: {}", file_name);
                offset += fname_size as usize;
                // extra field (variable size)
                log::debug!("拡張フィールド位置: {}", offset);
                offset += ex_length as usize;
                // file entry
                log::debug!("ファイルエントリ位置: {}", offset);
                let data_offset = offset;
                offset += csize as usize;

                // compress type
                let ctype = match comp {
                    0 => CompressionType::Uncompress,
                    8 => CompressionType::Deflate,
                    9 => CompressionType::Deflate64,
                    _ => CompressionType::Unsupported,
                };

                // add file info
                if csize > 0 {
                    files.push(MemberFile {
                        filepath: file_name.to_string(),
                        filename: file_name.to_string(),
                        offset: data_offset as u64,
                        size: csize as u64,
                        fsize: ucsize as u64,
                        ctype: ctype,
                    });
                }
            }
        }

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

fn read_comressed_data(buf : &Vec<u8>, offset : u64, size : u64) -> Vec<u8> {
    println!("compressed");
    let src: &[u8] = &buf[offset as usize..offset as usize +size as usize].to_owned();
    let mut deflater = DeflateDecoder::new(src);
    let mut data = Vec::new();
    deflater.read_to_end(&mut data);

    data
}


// pub fn read_rar_from_file(filename : &str, files : &mut Vec<MemberFile>) -> Result<(), Box<dyn std::error::Error>> {
//     let mut file = File::open(filename)?;
//     let mut buf = Vec::new();
//     let _ = file.read_to_end(&mut buf)?;

//     Rar5Reader::read_archive(&buf, files)
// }

fn check_zipsign(data: &[u8]) -> ArchiveResult<(usize, bool)> {
    if data.len() < 4 {
        return Ok((0, false));
    }

    if &data[0..4] == [0x50, 0x4B, 0x03, 0x04] {
        return Ok((0, true));
    }

    Ok((0, false))
}

// [Volume header] => total 7 bytes
//  header_crc    2 bytes
//  header_type   1 byte
//  header_flags  2 bytes
//  header_size   2 bytes
fn check_headertype(data: &[u8], pos: usize) -> (u8, u16, u16) {
    let mut offset : usize = pos;
    let _vintlen : u8 = 0;

    let htype : u8;
    let hflags : u16;
    let hsize : u16;

    if data.len() >= offset + 7 {
        // skip crc
        offset += 2;

        // header type
        htype = data[offset];
        offset += 1;

        // header flags
        hflags = (data[offset] as u16) << 8 | (data[offset] as u16);
        offset += 2;

        // header size
        hsize = (data[offset+1] as u16) << 8 | (data[offset] as u16);
        offset += 2;

    } else {
        htype = 0;
        hflags = 0;
        hsize = 0;
    }

    println!("DEBUG: Header (type:{:#02x}, flags:{:#02x}, size:{})", htype, hflags, hsize);

    (htype, hflags, hsize)
}

