use regex::{Regex, Captures, Replacer};
use log::{info, debug};

use crate::archive_reader::MemberFile;

struct PaddProc;

impl Replacer for PaddProc {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        dst.push_str(&format!("{x:0>30}", x = &caps[0]));
    }
}

pub fn sort_filename(files: &mut Vec<MemberFile>) {
    // 数字パディング用の正規表現を作成
    let re = match Regex::new(r"(\d+)") {
        Ok(regex) => regex,
        Err(e) => {
            eprintln!("正規表現の作成に失敗: {}", e);
            return; // ソートせずに終了
        }
    };

    files.sort_by(|a, b| {
        let mod_a = re.replace_all(&a.filepath, PaddProc);
        let mod_b = re.replace_all(&b.filepath, PaddProc);
        mod_a.to_lowercase().cmp(&mod_b.to_lowercase())
    });
    
    log::info!("ファイルをソートしました: {}件", files.len());
    for f in files {
        log::debug!("ファイル: {} (offset: {}, size: {}, fsize: {})", 
               f.filepath, f.offset, f.size, f.fsize);
    }
}
