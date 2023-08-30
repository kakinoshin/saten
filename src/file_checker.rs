pub enum FileType {
    Zip,
    Rar5,
    Rar4,
    Unsupported
}

pub fn check_file_type(buf : &Vec<u8>) -> FileType {
    if check_rar5(buf) {
        FileType::Rar5
    } else if check_rar4(buf) {
        FileType::Rar4
    } else if check_zip(buf) {
        FileType::Zip
    } else {
        FileType::Unsupported
    }
}

fn check_rar5(data : &Vec<u8>) -> bool {
    // RAR 5.0: 0x52 0x61 0x72 0x21 0x1A 0x07 0x01 0x00
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
                result = true;
                break;
            }
        }
    }

    result
}

fn check_rar4(data : &Vec<u8>) -> bool {
    // RAR 4.0: 0x52 0x61 0x72 0x21 0x1A 0x07 0x00
    let mut result : bool = false;

    for (i, d) in data.iter().enumerate() {
        if *d == 0x52 as u8 {
            if data[i+1] == 0x61 &&
               data[i+2] == 0x72 &&
               data[i+3] == 0x21 &&
               data[i+4] == 0x1A &&
               data[i+5] == 0x07 &&
               data[i+6] == 0x00 {
                result = true;
                break;
            }
        }
    }

    result
}

// ZIP file header 0x504B0304
fn check_zip(data : &Vec<u8>) -> bool {
    let mut result : bool = false;

    for (i, d) in data.iter().enumerate() {
        if *d == 0x50 as u8 {
            if data[i+1] == 0x4B &&
               data[i+2] == 0x03 &&
               data[i+3] == 0x04 {
                result = true;
                break;
            }
        }
    }

    result
}
