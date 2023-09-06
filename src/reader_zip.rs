// use std::fs::File;
// use std::io::Read;

use crate::archive_reader::ArcReader;
use crate::archive_reader::MemberFile;

pub struct ZipReader {
    buf: Vec<u8>,
    files: Vec<MemberFile>,
}

impl ArcReader for ZipReader {
    fn read_archive(buf : &Vec<u8>, files : &mut Vec<MemberFile>) -> Result<(), Box<dyn std::error::Error>> {
        let mut offset : usize = 0;

        // local file header signature     4 bytes  (0x04034b50)
        let (pos, is_sign) = check_zipsign(&buf);
        println!("signature pos : {:?}", pos);

        if is_sign {
            loop {
                if buf.len() <= offset + 30 {    // size of heaeder
                    break;
                }
                println!("DEBUG: block start pos: {:?}", offset);

                // local file header signature     4 bytes  (0x04034b50)
                if buf[offset] == 0x50 &&
                   buf[offset+1] == 0x4B && 
                   buf[offset+2] == 0x03 && 
                   buf[offset+3] == 0x04 {
                    println!("signature pos : {:?}", offset);
                } else {
                    println!("!!! no signature !!!");
                    break;
                }
                offset += 4;

                // version needed to extract       2 bytes
                let ver = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // general purpose bit flag        2 bytes
                let gpflag = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // compression method              2 bytes
                let comp = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // last mod file time              2 bytes
                let file_time = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // last mod file date              2 bytes
                let file_date = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // crc-32                          4 bytes
                let crc32 = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                offset += 4;
                // compressed size                 4 bytes
                let csize = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                println!("DEBUG: compressed size: {:?}", csize);
                offset += 4;
                // uncompressed size               4 bytes
                let ucsize = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                println!("DEBUG: uncompressed size: {:?}", ucsize);
                offset += 4;
                // file name length                2 bytes
                let fname_size = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // extra field length              2 bytes               
                let ex_length = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                offset += 2;
                // file name (variable size)
                let file_name = std::str::from_utf8(&buf[offset..(offset+fname_size as usize)]).unwrap();
                println!("{}", file_name);
                offset += fname_size as usize;
                // extra field (variable size)
                println!("DEBUG: extra field: {:?}", offset);
                offset += ex_length as usize;
                // file entry
                println!("DEBUG: file entry: {:?}", offset);
                let data_offset = offset;
                offset += csize as usize;

                // add file info
                if csize > 0 {
                    files.push(MemberFile {
                        filepath: file_name.to_string(),
                        filename: file_name.to_string(),
                        offset: data_offset as u64,
                        size: csize as u64,
                        fsize: ucsize as u64,
                    });
                }
            }
        }

        Ok(())
    }

    fn read_data(buf : &Vec<u8>, offset : u64, size : u64) -> Vec<u8> {
        buf[offset as usize..offset as usize +size as usize].to_owned()
    }
}

// pub fn read_rar_from_file(filename : &str, files : &mut Vec<MemberFile>) -> Result<(), Box<dyn std::error::Error>> {
//     let mut file = File::open(filename)?;
//     let mut buf = Vec::new();
//     let _ = file.read_to_end(&mut buf)?;

//     Rar5Reader::read_archive(&buf, files)
// }

fn check_zipsign(data : &Vec<u8>) -> (usize, bool) {
    let mut pos : usize = 0;
    let mut result : bool = false;

    for (i, d) in data.iter().enumerate() {
        if *d == 0x50 as u8 {
            if data[i+1] == 0x4b &&
               data[i+2] == 0x03 &&
               data[i+3] == 0x04 {
                pos = i;
                result = true;
                break;
            }
        }
    }

    (pos, result)
}

// [Volume header] => total 7 bytes
//  header_crc    2 bytes
//  header_type   1 byte
//  header_flags  2 bytes
//  header_size   2 bytes
fn check_headertype(data : &Vec<u8>, pos : usize) -> (u8, u16, u16) {
    let mut offset : usize = pos;
    let mut vintlen : u8 = 0;

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

