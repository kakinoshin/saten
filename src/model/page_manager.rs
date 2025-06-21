use log::{debug, info};
use crate::model::app_state::{AppState, DisplayMode};

pub struct PageManager;

impl PageManager {
    pub fn new() -> Self {
        Self
    }

    /// 次のページに移動
    pub fn next_page(state: &mut AppState) {
        match state.display_mode {
            DisplayMode::Single => {
                if state.current_file_index + 1 < state.total_files {
                    state.current_file_index += 1;
                    debug!("次のページに移動しました: {}/{}", state.current_file_index, state.total_files);
                }
            }
            DisplayMode::Double => {
                if state.current_file_index + 2 < state.total_files {
                    state.current_file_index += 2;
                    debug!("次の見開きページに移動しました: {}/{}", state.current_file_index, state.total_files);
                }
            }
        }
    }

    /// 前のページに移動
    pub fn previous_page(state: &mut AppState) {
        match state.display_mode {
            DisplayMode::Single => {
                if state.current_file_index > 0 {
                    state.current_file_index -= 1;
                    debug!("前のページに戻しました: {}/{}", state.current_file_index, state.total_files);
                }
            }
            DisplayMode::Double => {
                if state.current_file_index >= 2 {
                    state.current_file_index -= 2;
                    debug!("前の見開きページに戻しました: {}/{}", state.current_file_index, state.total_files);
                }
            }
        }
    }

    /// 次のファイルに移動（1つずつ）
    pub fn next_file(state: &mut AppState) {
        if state.current_file_index + 1 < state.total_files {
            state.current_file_index += 1;
            debug!("次のファイルに移動しました: {}/{}", state.current_file_index, state.total_files);
        }
    }

    /// 前のファイルに移動（1つずつ）
    pub fn previous_file(state: &mut AppState) {
        if state.current_file_index > 0 {
            state.current_file_index -= 1;
            debug!("前のファイルに戻しました: {}/{}", state.current_file_index, state.total_files);
        }
    }

    /// 最初のページに移動
    pub fn goto_first_page(state: &mut AppState) {
        state.current_file_index = 0;
        info!("最初のページに移動しました");
    }

    /// 最後のページに移動
    pub fn goto_last_page(state: &mut AppState) {
        if state.total_files > 0 {
            match state.display_mode {
                DisplayMode::Single => {
                    state.current_file_index = state.total_files - 1;
                }
                DisplayMode::Double => {
                    // 偶数ページから開始するように調整
                    if state.total_files >= 2 {
                        state.current_file_index = if state.total_files % 2 == 0 {
                            state.total_files - 2
                        } else {
                            state.total_files - 1
                        };
                    } else {
                        state.current_file_index = 0;
                    }
                }
            }
            info!("最後のページに移動しました");
        }
    }

    /// 指定ページに移動
    pub fn goto_page(state: &mut AppState, page_number: usize) {
        match state.display_mode {
            DisplayMode::Single => {
                if page_number < state.total_files {
                    state.current_file_index = page_number;
                    info!("ページ {} に移動しました", page_number + 1);
                }
            }
            DisplayMode::Double => {
                let adjusted_page = if page_number % 2 == 0 { page_number } else { page_number - 1 };
                if adjusted_page < state.total_files {
                    state.current_file_index = adjusted_page;
                    info!("見開きページ {} に移動しました", adjusted_page + 1);
                }
            }
        }
    }

    /// 現在のページ番号を取得（1ベース）
    pub fn get_current_page_number(state: &AppState) -> usize {
        state.current_file_index + 1
    }

    /// 総ページ数を取得
    pub fn get_total_pages(state: &AppState) -> usize {
        state.total_files
    }

    /// 次のページが存在するかチェック
    pub fn has_next_page(state: &AppState) -> bool {
        match state.display_mode {
            DisplayMode::Single => state.current_file_index + 1 < state.total_files,
            DisplayMode::Double => state.current_file_index + 2 < state.total_files,
        }
    }

    /// 前のページが存在するかチェック
    pub fn has_previous_page(state: &AppState) -> bool {
        match state.display_mode {
            DisplayMode::Single => state.current_file_index > 0,
            DisplayMode::Double => state.current_file_index >= 2,
        }
    }

    /// 表示モードを変更
    pub fn set_display_mode(state: &mut AppState, mode: DisplayMode) {
        let old_mode = format!("{:?}", state.display_mode);
        state.display_mode = mode;
        let new_mode = format!("{:?}", state.display_mode);
        info!("表示モードを {} から {} に変更しました", old_mode, new_mode);

        // モード変更時にページ位置を調整
        match state.display_mode {
            DisplayMode::Double => {
                // ダブルページモードでは偶数インデックスから開始
                if state.current_file_index % 2 != 0 && state.current_file_index > 0 {
                    state.current_file_index -= 1;
                }
            }
            DisplayMode::Single => {
                // シングルページモードでは特に調整不要
            }
        }
    }

    /// 回転モードの切り替え
    pub fn toggle_rotate_mode(state: &mut AppState) {
        state.rotate_mode = !state.rotate_mode;
        info!("回転モード: {}", if state.rotate_mode { "ON" } else { "OFF" });
    }

    /// ページ情報の文字列表現を取得
    pub fn get_page_info_string(state: &AppState) -> String {
        match state.display_mode {
            DisplayMode::Single => {
                format!("{} / {}", state.current_file_index + 1, state.total_files)
            }
            DisplayMode::Double => {
                let end_index = (state.current_file_index + 2).min(state.total_files);
                if end_index > state.current_file_index + 1 {
                    format!("{}-{} / {}", 
                        state.current_file_index + 1, 
                        end_index, 
                        state.total_files)
                } else {
                    format!("{} / {}", state.current_file_index + 1, state.total_files)
                }
            }
        }
    }
}

impl Default for PageManager {
    fn default() -> Self {
        Self::new()
    }
}
