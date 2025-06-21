// use std::fs::File;
// use std::io::Read;
use encoding_rs;

use crate::archive_reader::ArcReader;
use crate::archive_reader::MemberFile;
use crate::archive_reader::CompressionType;

pub struct Rar4Reader {
    buf: Vec<u8>,
    files: Vec<MemberFile>,
}

impl ArcReader for Rar4Reader {
    fn new() -> Self {
        Self {
            buf : Vec::new(),
            files : Vec::new(),
        }
    }

    fn read_archive(buf : &Vec<u8>, files : &mut Vec<MemberFile>) -> Result<(), Box<dyn std::error::Error>> {
        let mut offset : usize = 0;

        let (pos, is_sign) = check_rarsign(&buf);
        println!("signature pos : {:?}", pos);

        if is_sign {
            offset += pos + 7;  // skip signature

            loop {
                if buf.len() <= offset + 7 {    // size of heaeder
                    break;
                }

                let htype: u8;
                let hflags: u16;
                let hsize: u16;
                (htype, hflags, hsize) = check_headertype(&buf, offset);
                println!("header type : {:#02x}", htype);
                offset += 7;

                if hsize == 0 {
                    println!("Invalid header size");
                    break;
                }

                if htype == 0x72 {      // MARK_HEAD (0x72)
                    println!("Not supported header type ({})", htype);
                    break;
                } else if htype == 0x73 {  // MAIN_HEAD (0x73)
                    println!("DEBUG: [MAIN_HEAD] - 0x73");
                    println!("DEBUG: HighPosAv: {:#02x} {:#02x}", buf[offset], buf[offset+1]);
                    println!("DEBUG: PosAv: {:#02x} {:#02x} {:#02x} {:#02x}", buf[offset+2], buf[offset+3], buf[offset+4], buf[offset+5]);
                    offset += (hsize as usize) - 7;
                } else if htype == 0x74 {  // FILE_HEAD (0x74)
                    println!("DEBUG: [FILE_HEAD] - 0x74");
                    let psize = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                    println!("DEBUG: packed size:   {}", psize);
                    offset += 4;    // PackSize
                    let upsize = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                    println!("DEBUG: unpacked size: {}", upsize);
                    if psize != upsize {
                        println!("DEBUG: Compressed (Unsupported)");
                    }
                    offset += 4;    // UnpSize
                    offset += 1;    // HostOS
                    offset += 4;    // FileCRC
                    offset += 4;    // FileTime (mtime)
                    // UnpVer
                    let unpver = buf[offset] as u8;
                    offset += 1;
                    offset += 1;    // Method
                    let nsize = (buf[offset+1] as u16) << 8 | (buf[offset] as u16);
                    println!("DEBUG: filename size: {}", nsize);
                    offset += 2;    // NameSize
                    let fattr = (buf[offset+1] as u32) << 24 | (buf[offset+1] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                    println!("DEBUG: file attr: {:#04x}", fattr);
                    offset += 4;    // FileAttr
                    if (hflags & 0x0100) != 0 { //LHD_LARGE
                        println!("DEBUG: HighPackSize");
                        offset += 4;    // HighPackSize
                    }
                    if (hflags & 0x0100) != 0 { //LHD_LARGE
                        println!("DEBUG: HighUnpSize");
                        offset += 4;    // HighUnpSize
                    }
                    let mut endpos = offset+nsize as usize;
                    for i in offset..(offset+nsize as usize) {
                        if buf[i] == 0 {
                            endpos = i;
                            break;
                        }
                    }
                    println!("DEBUG: file name end position: {}", endpos);
                    //let file_name = std::str::from_utf8(&buf[offset..endpos]).unwrap();
                    let file_name = match std::str::from_utf8(&buf[offset..endpos]) {
                        Ok(fname) => fname,
                        Err(e) => {
                            println!("invalid file name error: {}", e.to_string());
                            let (res, _, _) = encoding_rs::UTF_8.decode(&buf[offset..endpos]);
                            //let (res, _, _) = encoding_rs::SHIFT_JIS.decode(&buf[offset..endpos]);
                            let text = res.into_owned();
                            println!("sjis: {}", text);
                            //let pt = &text;
                            //pt
                            "invalid_filename"
                        },
                    };
                    println!("{}", file_name);
                    offset += nsize as usize;   // FileName
                    if (hflags & 0x0400) != 0 { //LHD_SALT
                        println!("DEBUG: Salt");
                        offset += 4;    // Salt
                    }
                    if (hflags & 0x1000) != 0 { //LHD_EXTTIME
                        println!("DEBUG: ExtTime_Structure");
                        offset += 2;    // section flags (4bit x 4 sections)
                        offset += 4;    // ExtTime_Structure
                    }
                    //offset += 9;    // adjust (??)
                    let data_offset = offset as u64;
                    offset += psize as usize;   // Packaed Data
                    //println!("DEBUG: offset: {:#08x}", offset);

                    // compress type
                    let ctype = match unpver {
                        0 => CompressionType::Uncompress,
                        15 => CompressionType::Rar4,
                        20 => CompressionType::Rar4,
                        26 => CompressionType::Rar4,
                        29 => CompressionType::Rar4,
                        36 => CompressionType::Rar4,
                        _ => CompressionType::Unsupported,
                    };

                    // add file info
                    if (fattr & 0x20) != 0 {
                        files.push(MemberFile {
                            filepath: file_name.to_string(),
                            filename: file_name.to_string(),
                            offset: data_offset,
                            size: psize as u64,
                            fsize: upsize as u64,
                            ctype: ctype,
                        });
                    }
                } else if htype == 0x75	{   // COMM_HEAD (0x75)
                    println!("Not supported header type ({})", htype);
                    break;
                } else if htype == 0x76	{   // AV_HEAD (0x76)
                    println!("Not supported header type ({})", htype);
                    break;
                } else if htype == 0x77	{   // SUB_HEAD (0x77)
                    println!("Not supported header type ({})", htype);
                    break;
                } else if htype == 0x78	{   // PROTECT_HEAD (0x78)
                    println!("Not supported header type ({})", htype);
                    break;
                } else if htype == 0x79	{   // SIGN_HEAD (0x79)
                    println!("Not supported header type ({})", htype);
                    break;
                } else if htype == 0x7a {   // NEWSUB_HEAD (0x7a)
                    println!("DEBUG: [NEWSUB_HEAD] - 0x7a");
                    let newsub_size = (buf[offset+3] as u32) << 24 | (buf[offset+2] as u32) << 16 | (buf[offset+1] as u32) << 8 | (buf[offset] as u32);
                    println!("DEBUG: Size: {}", newsub_size);
                    offset += (hsize as usize) - 7; // skip header
                    offset += newsub_size as usize; // skip newsub body
                } else if htype == 0x7b {   // ENDARC_HEAD (0x7b)
                    println!("Not supported header type ({})", htype);
                    break;
                } else {
                    println!("Not supported header type ({})", htype);
                    break;
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

fn check_rarsign(data : &Vec<u8>) -> (usize, bool) {
    let mut pos : usize = 0;
    let mut result : bool = false;

    for (i, d) in data.iter().enumerate() {
        if *d == 0x52 as u8 {
            // println!("{:#04X}", data[i+1]);
            if data[i+1] == 0x61 &&
               data[i+2] == 0x72 &&
               data[i+3] == 0x21 &&
               data[i+4] == 0x1A &&
               data[i+5] == 0x07 &&
               data[i+6] == 0x00 {
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
        hflags = (data[offset+1] as u16) << 8 | (data[offset] as u16);
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

