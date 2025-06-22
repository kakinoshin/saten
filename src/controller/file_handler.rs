use std::path::PathBuf;
use log::{info, error, debug};
use iced::Task;

use crate::model::app_state::AppState;
use crate::model::archive_manager::ArchiveManager;
use crate::archive_reader::MemberFile;
use crate::controller::app_controller::Message;

pub struct FileHandler;

impl FileHandler {
    pub fn new() -> Self {
        Self
    }

    /// ファイルドロップイベントを処理
    pub fn handle_file_drop(
        state: &mut AppState,
        path: PathBuf
    ) -> Task<Message> {
        debug!("ファイルがドロップされました: {:?}", path);
        
        // ファイルパスを設定
        state.set_file_path(path.clone());

        // 非同期でファイルを読み込み
        Task::perform(
            async move {
                Self::load_file_async(path).await
            },
            Message::FileLoaded
        )
    }

    /// 非同期でファイルを読み込み
    async fn load_file_async(
        path: PathBuf
    ) -> Result<(Vec<u8>, Vec<MemberFile>), String> {
        // ファイル拡張子チェック
        if !Self::is_supported_file(&path) {
            return Err("サポートされていないファイル形式です".to_string());
        }

        // ファイル読み込み
        let buffer = match ArchiveManager::load_archive_file(&path) {
            Ok(buf) => buf,
            Err(e) => {
                return Err(format!("ファイル読み込みエラー: {}", e));
            }
        };

        // アーカイブ処理
        let files = match ArchiveManager::process_archive(&buffer) {
            Ok(files) => files,
            Err(e) => {
                return Err(format!("アーカイブ処理エラー: {}", e));
            }
        };

        // 画像ファイルのフィルタリング
        let image_files = Self::filter_image_files(files);
        
        if image_files.is_empty() {
            return Err("アーカイブ内に画像ファイルが見つかりません".to_string());
        }

        Ok((buffer, image_files))
    }

    /// サポートされているファイル形式かチェック
    fn is_supported_file(path: &PathBuf) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return matches!(ext_lower.as_str(), "rar" | "zip" | "cbr" | "cbz");
            }
        }
        false
    }

    /// 画像ファイルのみをフィルタリング
    fn filter_image_files(files: Vec<MemberFile>) -> Vec<MemberFile> {
        files.into_iter()
            .filter(|file| Self::is_image_file(&file.filename))
            .collect()
    }

    /// 画像ファイルかどうかをチェック
    fn is_image_file(filename: &str) -> bool {
        let filename_lower = filename.to_lowercase();
        
        // 一般的な画像ファイル拡張子
        let image_extensions = [
            ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", 
            ".tiff", ".tif", ".ico", ".svg", ".avif"
        ];

        image_extensions.iter()
            .any(|ext| filename_lower.ends_with(ext))
    }

    /// ファイルサイズを取得
    pub fn get_file_size(path: &PathBuf) -> Result<u64, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len())
    }

    /// ファイル情報を取得
    pub fn get_file_info(path: &PathBuf) -> Result<FileInfo, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(FileInfo {
            name: file_name,
            size: metadata.len(),
            is_directory: metadata.is_dir(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
        })
    }

    /// 複数ファイルのドロップ処理
    pub fn handle_multiple_files_drop(
        state: &mut AppState,
        paths: Vec<PathBuf>
    ) -> Task<Message> {
        if paths.is_empty() {
            return Task::none();
        }

        // 最初のサポートされているファイルを選択
        for path in paths {
            if Self::is_supported_file(&path) {
                return Self::handle_file_drop(state, path);
            }
        }

        // サポートされているファイルが見つからない場合
        error!("サポートされているファイルが見つかりませんでした");
        Task::perform(
            async { "サポートされているファイルがありません".to_string() },
            Message::ShowError
        )
    }

    /// ファイル読み込み進捗の更新（将来の拡張用）
    pub fn update_loading_progress(progress: f32) {
        debug!("読み込み進捗: {:.1}%", progress * 100.0);
        // 将来的には進捗バーの更新などを行う
    }

    /// ファイル読み込みキャンセル（将来の拡張用）
    pub fn cancel_file_loading() -> Task<Message> {
        info!("ファイル読み込みをキャンセルしました");
        Task::none()
    }

    /// 最近使用したファイルの管理（将来の拡張用）
    pub fn add_to_recent_files(path: &PathBuf) {
        debug!("最近使用したファイルに追加: {:?}", path);
        // 将来的には最近使用したファイルのリストを管理
    }

    /// ファイルの妥当性検証
    pub fn validate_file(path: &PathBuf) -> Result<(), String> {
        // ファイルの存在チェック
        if !path.exists() {
            return Err("ファイルが存在しません".to_string());
        }

        // ファイルサイズチェック
        match Self::get_file_size(path) {
            Ok(size) => {
                if size == 0 {
                    return Err("ファイルサイズが0です".to_string());
                }
                if size > 1024 * 1024 * 1024 { // 1GB制限
                    return Err("ファイルサイズが大きすぎます（1GB制限）".to_string());
                }
            }
            Err(_) => {
                return Err("ファイル情報の取得に失敗しました".to_string());
            }
        }

        // 拡張子チェック
        if !Self::is_supported_file(path) {
            return Err("サポートされていないファイル形式です".to_string());
        }

        Ok(())
    }
}

impl Default for FileHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// ファイル情報構造体
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub is_directory: bool,
    pub modified: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

impl FileInfo {
    /// ファイルサイズを人間が読みやすい形式で取得
    pub fn size_string(&self) -> String {
        let size = self.size as f64;
        
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }

    /// 更新日時を文字列で取得
    pub fn modified_string(&self) -> String {
        match &self.modified {
            Some(time) => {
                match time.duration_since(std::time::UNIX_EPOCH) {
                    Ok(duration) => {
                        // 簡易的な日時表示（実際にはchronoクレートなどを使用推奨）
                        format!("{} seconds since epoch", duration.as_secs())
                    }
                    Err(_) => "Unknown".to_string(),
                }
            }
            None => "Unknown".to_string(),
        }
    }
}
