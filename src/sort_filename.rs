use regex::{Regex, Captures, Replacer};

use crate::archive_reader::MemberFile;

struct PaddProc;

impl Replacer for PaddProc {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        dst.push_str(&format!("{x:0>30}", x = &caps[0]));
    }
}

pub fn sort_filename(files : &mut Vec<MemberFile>) {
    files.sort_by(|a, b| {
        let re = Regex::new(r"(\d+)").unwrap();
        let mod_a = re.replace_all(&a.filepath, PaddProc);
        let mod_b = re.replace_all(&b.filepath, PaddProc);
            mod_a.to_lowercase().cmp(&mod_b.to_lowercase())
    });
    for f in files {
        println!("{}/{}/{}/{}", f.filepath, f.offset, f.size, f.fsize);
    }
}
