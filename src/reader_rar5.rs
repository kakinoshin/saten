// use std::fs::File;
// use std::io::Read;

use crate::archive_reader::ArcReader;
use crate::archive_reader::MemberFile;

pub struct Rar5Reader {
    buf: Vec<u8>,
    files: Vec<MemberFile>,
}

impl ArcReader for Rar5Reader {
    fn read_archive(buf : &Vec<u8>, files : &mut Vec<MemberFile>) -> Result<(), Box<dyn std::error::Error>> {
        let mut offset : usize = 0;
    
        let (pos, is_sign) = check_rarsign(&buf);
        println!("signature pos : {:?}", pos);
        
        if is_sign {
            offset += pos + 8;
            let htype = check_headertype(&buf, offset);
            println!("header type : {:?}", htype);
    
            if htype == 1 {
                // read Main archive header
                let hsize;
                hsize = check_header_mainarchive(&buf, offset);
                println!("header size : {:?}", hsize);
                offset += hsize;
                println!("offset : {:?}", offset);
    
                // read next block
                loop {
                    let btype = check_headertype(&buf, offset);
                    println!("header type : {:?}", btype);
                    if btype == 2 {
                        println!("== File header ==");
                        offset += check_header_file(&buf, offset, files);
                    } else if btype == 3 {
                        println!("== Service header ==");
                        offset += check_header_service(&buf, offset);
                    } else {
                        println!("there is no file header");
                        break;
                    }
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

// return (u64, u8)
//  data : u64 -> vint data
//  size : u8  -> size of data (bytes), size == MAX(255) means failed to read vint value
fn read_vint(data : &Vec<u8>, pos : usize) -> (u64, u8) {
    let mut offset = 0 as u8;
    let mut val = 0 as u64;
    let mut shift = 0;
    loop {
        if data.len() < pos + offset as usize {
            offset = std::u8::MAX;
            break;
        } else {
            let d = data[pos + offset as usize] as u64;
            val = ((d & 0x7F) << shift) | val;
            if d & 0x80 != 0x80 {
                break; 
            }
            offset += 1;
            shift += 7;
        }

        // if necessary need implementation of negative case
        assert!(offset < 10);
    }

    (val, offset + 1)
}

fn check_rarsign(data : &Vec<u8>) -> (usize, bool) {
    // RAR 5.0: 0x52 0x61 0x72 0x21 0x1A 0x07 0x01 0x00
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
               data[i+6] == 0x01 &&
               data[i+7] == 0x00 {
                pos = i;
                result = true;
                break;
            } else if data[i+1] == 0x61 &&
                        data[i+2] == 0x72 &&
                        data[i+3] == 0x21 &&
                        data[i+4] == 0x1A &&
                        data[i+5] == 0x07 &&
                        data[i+6] == 0x00 {
                // RAR 4.x
                pos = i;
                result = true;
                break;
            }
        }
    }

    (pos, result)
}

fn check_headertype(data : &Vec<u8>, pos : usize) -> u64 {
    let mut offset : usize = pos;
    let mut vintlen : u8 = 0;

    let htype : u64;
    let hsize : u64;

    if data.len() >= offset + 6 {
        // skip crc
        offset += 4;

        // header size
        (hsize, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::u64::MAX;
        }
        offset += vintlen as usize;

        // header type
        (htype, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::u64::MAX;
        }
        offset += vintlen as usize;
    } else {
        htype = 0;
        hsize = 0;
    }

    htype
}

fn check_header_mainarchive(data : &Vec<u8>, pos : usize) -> usize {
    let mut offset : usize = pos;
    let mut vintlen : u8 = 0;
    let mut headerlen : usize = 0;

    let htype : u64;
    let hsize : u64;

    if data.len() >= pos + 6 {
        // skip crc
        offset += 4;

        // header size
        (hsize, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;

        // calc main archive header size
        headerlen = 4 + vintlen as usize + hsize as usize;

        // header type
        (htype, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;
    } else {
        htype = 0;
        hsize = 0;
    }

    if htype != 1 || data.len() < pos + hsize as usize {
        return std::usize::MAX;
    }

    // Header flags
    let hflag;
    (hflag, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;

    // Extra area size
    let extra_size;
    (extra_size, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;

    // Archive flags
    let aflag;
    (aflag, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;

    let is_extra;
    let is_data;
    let is_unknown;
    let is_continue_blk_fromprev;
    let is_continue_blk_tonext;
    let is_file_blk;
    let is_preserve_child;
    if hflag & 0x01 == 0x01 {
        is_extra = true;
    } else {
        is_extra = false;
    }
    if hflag & 0x02 == 0x02 {
        is_data = true;
    } else {
        is_data = false;
    }
    if hflag & 0x04 == 0x04 {
        is_unknown = true;
    } else {
        is_unknown = false;
    }
    if hflag & 0x08 == 0x08 {
        is_continue_blk_fromprev = true;
    } else {
        is_continue_blk_fromprev = false;
    }
    if hflag & 0x10 == 0x10 {
        is_continue_blk_tonext = true;
    } else {
        is_continue_blk_tonext = false;
    }
    if hflag & 0x20 == 0x20 {
        is_file_blk = true;
    } else {
        is_file_blk = false;
    }
    if hflag & 0x40 == 0x40 {
        is_preserve_child = true;
    } else {
        is_preserve_child = false;
    }

    println!("== Header Flags ==");
    println!("Extra area             = {:?}", is_extra);
    println!("Data area              = {:?}", is_data);
    println!("Unknown area           = {:?}", is_unknown);
    println!("Continue from previous = {:?}", is_continue_blk_fromprev);
    println!("Continue to next       = {:?}", is_continue_blk_tonext);
    println!("File block             = {:?}", is_file_blk);
    println!("Preserve Child block   = {:?}", is_preserve_child);

    if is_extra {
        println!("{:?}", extra_size)
    }

    let is_multivolume = aflag & 0x01 == 0x01;
    let is_notfirst = aflag & 0x02 == 0x02;
    let is_solid_archive = aflag & 0x04 == 0x04;
    let is_recovery_record = aflag & 0x08 == 0x08;
    let is_lcoked = aflag & 0x10 == 0x10;

    println!("== Archive Flags ==");
    println!("Multiple Volume = {:?}", is_multivolume);
    println!("Not First Block = {:?}", is_notfirst);
    println!("Solid Archive   = {:?}", is_solid_archive);
    println!("Recovery Record = {:?}", is_recovery_record);
    println!("Locked Archive  = {:?}", is_lcoked);

    // Volume number
    let volnum;
    if is_notfirst {
        (volnum, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;
    } else {
        volnum = 0;
    }

    println!("== Volume information ==");
    println!("Volume number = {:?}", volnum + 1);

    // return total header size
    headerlen
}

fn check_header_file(data : &Vec<u8>, pos : usize, files : &mut Vec<MemberFile>) -> usize {
    let mut offset : usize = pos;
    let mut vintlen : u8 = 0;
    let mut headerlen : usize = 0;

    let htype : u64;
    let hsize : u64;

    if data.len() >= pos + 6 {
        // skip crc
        offset += 4;

        // header size
        (hsize, vintlen) = read_vint(&data, offset);
        println!("debug: hsize = {:?}, vintlen = {:?}", hsize, vintlen);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;

        // calc main archive header size
        headerlen = 4 + vintlen as usize + hsize as usize;

        // header type
        (htype, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;
    } else {
        htype = 0;
        hsize = 0;
    }

    if htype != 2 || data.len() < pos + hsize as usize {
        return std::usize::MAX;
    }

    // Header flags
    let hflag;
    (hflag, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;

    let is_extra;
    let is_data;
    let is_unknown;
    let is_continue_blk_fromprev;
    let is_continue_blk_tonext;
    let is_file_blk;
    let is_preserve_child;
    if hflag & 0x01 == 0x01 {
        is_extra = true;
    } else {
        is_extra = false;
    }
    if hflag & 0x02 == 0x02 {
        is_data = true;
    } else {
        is_data = false;
    }
    if hflag & 0x04 == 0x04 {
        is_unknown = true;
    } else {
        is_unknown = false;
    }
    if hflag & 0x08 == 0x08 {
        is_continue_blk_fromprev = true;
    } else {
        is_continue_blk_fromprev = false;
    }
    if hflag & 0x10 == 0x10 {
        is_continue_blk_tonext = true;
    } else {
        is_continue_blk_tonext = false;
    }
    if hflag & 0x20 == 0x20 {
        is_file_blk = true;
    } else {
        is_file_blk = false;
    }
    if hflag & 0x40 == 0x40 {
        is_preserve_child = true;
    } else {
        is_preserve_child = false;
    }

    println!("== Header Flags ==");
    println!("Extra area             = {:?}", is_extra);
    println!("Data area              = {:?}", is_data);
    println!("Unknown area           = {:?}", is_unknown);
    println!("Continue from previous = {:?}", is_continue_blk_fromprev);
    println!("Continue to next       = {:?}", is_continue_blk_tonext);
    println!("File block             = {:?}", is_file_blk);
    println!("Preserve Child block   = {:?}", is_preserve_child);

    // Extra area size
    let extra_size;
    if is_extra {
        (extra_size, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;
        println!("Extra area size = {:?}", extra_size)
    } else {
        extra_size = 0;
    }

    // Data size
    let data_size;
    if is_data {
        (data_size, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;
        println!("Data size = {:?}", data_size)
    } else {
        data_size = 0;
    }
    
    // File flag
    let fflag;
    (fflag, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;

    let is_dir;
    let is_unixtime;
    let is_crc32;
    let is_unknown_size;
    if fflag & 0x01 == 0x01 {
        is_dir = true;
    } else {
        is_dir = false;
    }
    if fflag & 0x02 == 0x02 {
        is_unixtime = true;
    } else {
        is_unixtime = false;
    }
    if fflag & 0x04 == 0x04 {
        is_crc32 = true;
    } else {
        is_crc32 = false;
    }
    if fflag & 0x08 == 0x08 {
        is_unknown_size = true;
    } else {
        is_unknown_size = false;
    }
    println!("== File Flags ==");
    println!("Directory file system object = {:?}", is_dir);
    println!("Unix Time field              = {:?}", is_unixtime);
    println!("CRC32                        = {:?}", is_crc32);
    println!("Unknown unpacked size        = {:?}", is_unknown_size);

    // Unpacked size
    let file_size;
    (file_size, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("Unpacked size = {:?}", file_size);

    // Attributes
    let file_attr;
    (file_attr, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("File Attrubutes = {:?}", file_attr);

    // mtime
    let file_mtime : u32;
    if is_unixtime {
        file_mtime = (data[offset] as u32) << 24 |
                     (data[offset+1] as u32) << 16 |
                     (data[offset+2] as u32) << 8 |
                     (data[offset+3] as u32);
        offset += 4;
    } else {
        file_mtime = 0;
    }
    println!("File mtime = {:#010X}", file_mtime);
    println!("File mtime = {:?}", file_mtime);

    // Data CRC32
    let file_crc32 : u32;
    if is_crc32 {
        file_crc32 = (data[offset] as u32) << 24 |
                     (data[offset+1] as u32) << 16 |
                     (data[offset+2] as u32) << 8 |
                     (data[offset+3] as u32);
        offset += 4;
    } else {
        file_crc32 = 0;
    }
    println!("File CRC32 = {:#010X}", file_crc32);

    // Compression information
    let file_comp;
    (file_comp, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("File Compression = {:#0X}(ver:{:#0X}/solid:{:#0X}/method:{:#0X}/dicsize:{:#0X})",
        file_comp,
        file_comp & 0x003f,
        (file_comp & 0x0040) >> 6,
        (file_comp & 0x0380) >> 7,
        (file_comp & 0x3c00) >> 10 );

    // Host OS (0:Windows/1:Unix)
    let file_hostos;
    (file_hostos, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("Host OS = {:?}", file_hostos);

    // Name length
    let file_namelen;
    (file_namelen, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("File name length = {:?}", file_namelen);

    // File Name
    let file_name = std::str::from_utf8(&data[offset..(offset+file_namelen as usize)]).unwrap();
    offset += file_namelen as usize;
    println!("File name = {:?}", file_name);
    
    // Extra area
    if is_extra {
        // skip Extra area
        offset += extra_size as usize;
    }
    println!("debug: pos = {:?}", pos);
    println!("debug: headerlen = {:?}", headerlen);
    println!("debug: offset = {:?}", offset);

    // Data area
    let data_offset : u64;
    if is_data {
        // skip Data area
        data_offset = offset as u64;
        offset += data_size as usize;
    } else {
        data_offset = 0;
    }
    println!("debug: offset = {:?}", offset);

    // add file info
    if is_data && !is_dir {
        files.push(MemberFile {
            filepath: file_name.to_string(),
            filename: file_name.to_string(),
            offset: data_offset,
            size: data_size,
            fsize: file_size,
        });
    }

    // return total header size
    //headerlen + data_size as usize
    offset - pos
}

fn check_header_service(data : &Vec<u8>, pos : usize) -> usize {
    let mut offset : usize = pos;
    let mut vintlen : u8 = 0;
    let mut headerlen : usize = 0;

    let htype : u64;
    let hsize : u64;

    if data.len() >= pos + 6 {
        // skip crc
        offset += 4;

        // header size
        (hsize, vintlen) = read_vint(&data, offset);
        println!("debug: hsize = {:?}, vintlen = {:?}", hsize, vintlen);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;

        // calc main archive header size
        headerlen = 4 + vintlen as usize + hsize as usize;

        // header type
        (htype, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;
    } else {
        htype = 0;
        hsize = 0;
    }

    if htype != 3 || data.len() < pos + hsize as usize {
        return std::usize::MAX;
    }

    // Header flags
    let hflag;
    (hflag, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;

    let is_extra;
    let is_data;
    let is_unknown;
    let is_continue_blk_fromprev;
    let is_continue_blk_tonext;
    let is_file_blk;
    let is_preserve_child;
    if hflag & 0x01 == 0x01 {
        is_extra = true;
    } else {
        is_extra = false;
    }
    if hflag & 0x02 == 0x02 {
        is_data = true;
    } else {
        is_data = false;
    }
    if hflag & 0x04 == 0x04 {
        is_unknown = true;
    } else {
        is_unknown = false;
    }
    if hflag & 0x08 == 0x08 {
        is_continue_blk_fromprev = true;
    } else {
        is_continue_blk_fromprev = false;
    }
    if hflag & 0x10 == 0x10 {
        is_continue_blk_tonext = true;
    } else {
        is_continue_blk_tonext = false;
    }
    if hflag & 0x20 == 0x20 {
        is_file_blk = true;
    } else {
        is_file_blk = false;
    }
    if hflag & 0x40 == 0x40 {
        is_preserve_child = true;
    } else {
        is_preserve_child = false;
    }

    println!("== Header Flags ==");
    println!("Extra area             = {:?}", is_extra);
    println!("Data area              = {:?}", is_data);
    println!("Unknown area           = {:?}", is_unknown);
    println!("Continue from previous = {:?}", is_continue_blk_fromprev);
    println!("Continue to next       = {:?}", is_continue_blk_tonext);
    println!("File block             = {:?}", is_file_blk);
    println!("Preserve Child block   = {:?}", is_preserve_child);

    // Extra area size
    let extra_size;
    if is_extra {
        (extra_size, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;
        println!("Extra area size = {:?}", extra_size)
    } else {
        extra_size = 0;
    }

    // Data size
    let data_size;
    if is_data {
        (data_size, vintlen) = read_vint(&data, offset);
        if vintlen == std::u8::MAX {
            return std::usize::MAX;
        }
        offset += vintlen as usize;
        println!("Data size = {:?}", data_size)
    } else {
        data_size = 0;
    }
    
    // File flag
    let fflag;
    (fflag, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;

    let is_dir;
    let is_unixtime;
    let is_crc32;
    let is_unknown_size;
    if fflag & 0x01 == 0x01 {
        is_dir = true;
    } else {
        is_dir = false;
    }
    if fflag & 0x02 == 0x02 {
        is_unixtime = true;
    } else {
        is_unixtime = false;
    }
    if fflag & 0x04 == 0x04 {
        is_crc32 = true;
    } else {
        is_crc32 = false;
    }
    if fflag & 0x08 == 0x08 {
        is_unknown_size = true;
    } else {
        is_unknown_size = false;
    }
    println!("== File Flags ==");
    println!("Directory file system object = {:?}", is_dir);
    println!("Unix Time field              = {:?}", is_unixtime);
    println!("CRC32                        = {:?}", is_crc32);
    println!("Unknown unpacked size        = {:?}", is_unknown_size);

    // Unpacked size
    let file_size;
    (file_size, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("Unpacked size = {:?}", file_size);

    // Attributes
    let file_attr;
    (file_attr, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("File Attrubutes = {:?}", file_attr);

    // mtime
    let file_mtime : u32;
    if is_unixtime {
        file_mtime = (data[offset] as u32) << 24 |
                     (data[offset+1] as u32) << 16 |
                     (data[offset+2] as u32) << 8 |
                     (data[offset+3] as u32);
        offset += 4;
    } else {
        file_mtime = 0;
    }
    println!("File mtime = {:#010X}", file_mtime);
    println!("File mtime = {:?}", file_mtime);

    // Data CRC32
    let file_crc32 : u32;
    if is_crc32 {
        file_crc32 = (data[offset] as u32) << 24 |
                     (data[offset+1] as u32) << 16 |
                     (data[offset+2] as u32) << 8 |
                     (data[offset+3] as u32);
        offset += 4;
    } else {
        file_crc32 = 0;
    }
    println!("File CRC32 = {:#010X}", file_crc32);

    // Compression information
    let file_comp;
    (file_comp, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("File Compression = {:#0X}", file_comp);

    // Host OS (0:Windows/1:Unix)
    let file_hostos;
    (file_hostos, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("Host OS = {:?}", file_hostos);

    // Name length
    let file_namelen;
    (file_namelen, vintlen) = read_vint(&data, offset);
    if vintlen == std::u8::MAX {
        return std::usize::MAX;
    }
    offset += vintlen as usize;
    println!("File name length = {:?}", file_namelen);

    // File Name
    let file_name = std::str::from_utf8(&data[offset..(offset+file_namelen as usize)]).unwrap();
    offset += file_namelen as usize;
    println!("File name = {:?}", file_name);
    
    // Extra area
    if is_extra {
        // skip Extra area
        offset += extra_size as usize;
    }
    println!("debug: pos = {:?}", pos);
    println!("debug: headerlen = {:?}", headerlen);
    println!("debug: offset = {:?}", offset);

    // Data area
    let data_offset : u64;
    if is_data {
        // skip Data area
        data_offset = offset as u64;
        offset += data_size as usize;
    } else {
        data_offset = 0;
    }
    println!("debug: offset = {:?}", offset);

    // return total header size
    //headerlen + data_size as usize
    offset - pos
}

// pub fn read_data_from_file(filename : &str, offset : u64, size : u64) -> Vec<u8> {
//     let mut file = match File::open(filename) {
//         Ok(f) => f,
//         Err(err) => panic!("file error: {}", err)
//     };
//     let mut buf = Vec::new();
//     let _ = file.read_to_end(&mut buf);

//     buf[offset as usize..offset as usize +size as usize].to_owned()
// }

