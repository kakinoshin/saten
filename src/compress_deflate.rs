use std::io::Read;
use flate2::read::DeflateDecoder;

pub fn uncomp_deflate(buf : &Vec<u8>, offset : u64, size : u64) -> Vec<u8> {
    println!("compressed");
    let src: &[u8] = &buf[offset as usize..offset as usize +size as usize].to_owned();
    let mut deflater = DeflateDecoder::new(src);
    let mut data = Vec::new();
    deflater.read_to_end(&mut data);

    data
}
