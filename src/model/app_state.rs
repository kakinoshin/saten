use std::path::PathBuf;
use crate::archive_reader::MemberFile;

#[derive(Debug, Default)]
pub enum DisplayMode {
    Single,
    #[default]
    Double,
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayMode::Single => write!(f, "Single"),
            DisplayMode::Double => write!(f, "Double"),
        }
    }
}

#[derive(Debug, Default)]
pub struct AppState {
    pub current_file_path: PathBuf,
    pub archive_files: Vec<MemberFile>,
    pub current_file_index: usize,
    pub total_files: usize,
    pub archive_buffer: Vec<u8>,
    pub display_mode: DisplayMode,
    pub rotate_mode: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// ファイルパスを設定
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.current_file_path = path;
    }

    /// アーカイブファイルリストを設定
    pub fn set_archive_files(&mut self, files: Vec<MemberFile>) {
        self.archive_files = files;
        self.total_files = self.archive_files.len();
        self.current_file_index = 0;
    }

    /// アーカイブバッファを設定
    pub fn set_archive_buffer(&mut self, buffer: Vec<u8>) {
        self.archive_buffer = buffer;
    }

    /// 現在のファイルインデックスを設定
    pub fn set_current_index(&mut self, index: usize) {
        if index < self.total_files {
            self.current_file_index = index;
        }
    }

    /// 表示モードを設定
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }

    /// 回転モードの切り替え
    pub fn toggle_rotate_mode(&mut self) {
        self.rotate_mode = !self.rotate_mode;
    }

    /// 次のページへ移動（ダブルページの場合は2つ進む）
    pub fn next_page(&mut self) {
        match self.display_mode {
            DisplayMode::Single => {
                if self.current_file_index + 1 < self.total_files {
                    self.current_file_index += 1;
                }
            }
            DisplayMode::Double => {
                if self.current_file_index + 2 < self.total_files {
                    self.current_file_index += 2;
                }
            }
        }
    }

    /// 前のページへ移動（ダブルページの場合は2つ戻る）
    pub fn previous_page(&mut self) {
        match self.display_mode {
            DisplayMode::Single => {
                if self.current_file_index > 0 {
                    self.current_file_index -= 1;
                }
            }
            DisplayMode::Double => {
                if self.current_file_index >= 2 {
                    self.current_file_index -= 2;
                }
            }
        }
    }

    /// 次のファイルへ移動（1つずつ）
    pub fn next_file(&mut self) {
        if self.current_file_index + 1 < self.total_files {
            self.current_file_index += 1;
        }
    }

    /// 前のファイルへ移動（1つずつ）
    pub fn previous_file(&mut self) {
        if self.current_file_index > 0 {
            self.current_file_index -= 1;
        }
    }

    /// 現在のファイルを取得
    pub fn current_file(&self) -> Option<&MemberFile> {
        self.archive_files.get(self.current_file_index)
    }

    /// 指定インデックスのファイルを取得
    pub fn get_file(&self, index: usize) -> Option<&MemberFile> {
        self.archive_files.get(index)
    }

    /// ファイルが読み込まれているかチェック
    pub fn has_files(&self) -> bool {
        !self.archive_files.is_empty()
    }

    /// ファイルパス文字列を取得
    pub fn file_path_string(&self) -> String {
        self.current_file_path
            .to_str()
            .unwrap_or("")
            .to_string()  // 所有データとして返す
    }

    /// アプリケーションをリセット（エラー時など）
    pub fn reset(&mut self) {
        self.archive_files.clear();
        self.archive_buffer.clear();
        self.current_file_index = 0;
        self.total_files = 0;
    }
}
