use log::{debug, info};
use iced::keyboard::{Event as KeyboardEvent, KeyCode};

use crate::model::app_state::{AppState, DisplayMode};
use crate::model::page_manager::PageManager;

pub struct KeyboardHandler;

impl KeyboardHandler {
    pub fn new() -> Self {
        Self
    }

    /// キーボードイベントを処理
    pub fn handle_keyboard_event(
        state: &mut AppState,
        event: KeyboardEvent
    ) {
        match event {
            KeyboardEvent::KeyPressed { key_code, modifiers } => {
                Self::handle_key_press(state, key_code, modifiers);
            }
            KeyboardEvent::KeyReleased { .. } => {
                // キーリリースイベントは現在未使用
            }
            _ => {}
        }
    }

    /// キー押下イベントの処理
    fn handle_key_press(
        state: &mut AppState,
        key_code: KeyCode,
        _modifiers: iced::keyboard::Modifiers
    ) {
        match key_code {
            // ページナビゲーション
            KeyCode::Left => {
                debug!("← キーが押されました");
                PageManager::next_page(state);
            }
            KeyCode::Right => {
                debug!("→ キーが押されました");
                PageManager::previous_page(state);
            }
            
            // ファイルナビゲーション（上下キー）
            KeyCode::Up => {
                debug!("↑ キーが押されました");
                PageManager::previous_file(state);
            }
            KeyCode::Down => {
                debug!("↓ キーが押されました");
                PageManager::next_file(state);
            }

            // 表示モード切り替え
            KeyCode::Key1 => {
                debug!("1 キーが押されました");
                PageManager::set_display_mode(state, DisplayMode::Single);
            }
            KeyCode::Key2 => {
                debug!("2 キーが押されました");
                PageManager::set_display_mode(state, DisplayMode::Double);
            }

            // 回転モード切り替え
            KeyCode::R => {
                debug!("R キーが押されました");
                PageManager::toggle_rotate_mode(state);
            }

            // ページジャンプ
            KeyCode::Home => {
                debug!("Home キーが押されました");
                PageManager::goto_first_page(state);
            }
            KeyCode::End => {
                debug!("End キーが押されました");
                PageManager::goto_last_page(state);
            }

            // ページ送り（Page Up/Down）
            KeyCode::PageUp => {
                debug!("Page Up キーが押されました");
                PageManager::previous_page(state);
            }
            KeyCode::PageDown => {
                debug!("Page Down キーが押されました");
                PageManager::next_page(state);
            }

            // スペースキー（ページ送り）
            KeyCode::Space => {
                debug!("Space キーが押されました");
                PageManager::next_page(state);
            }

            // Backspace（戻る）
            KeyCode::Backspace => {
                debug!("Backspace キーが押されました");
                PageManager::previous_page(state);
            }

            // その他のキー
            _ => {
                // 未定義のキーは無視
                debug!("未定義のキーが押されました: {:?}", key_code);
            }
        }
    }

    /// 修飾キーを考慮したキー処理
    pub fn handle_key_with_modifiers(
        state: &mut AppState,
        key_code: KeyCode,
        modifiers: iced::keyboard::Modifiers
    ) {
        if modifiers.shift() {
            Self::handle_shift_key_combination(state, key_code);
        } else if modifiers.control() {
            Self::handle_ctrl_key_combination(state, key_code);
        } else if modifiers.alt() {
            Self::handle_alt_key_combination(state, key_code);
        } else {
            Self::handle_key_press(state, key_code, modifiers);
        }
    }

    /// Shift + キーの組み合わせ処理
    fn handle_shift_key_combination(
        state: &mut AppState,
        key_code: KeyCode
    ) {
        match key_code {
            KeyCode::Left => {
                debug!("Shift + ← キーが押されました");
                // 大きくページを戻る（10ページ）
                for _ in 0..10 {
                    PageManager::previous_page(state);
                    if !PageManager::has_previous_page(state) {
                        break;
                    }
                }
            }
            KeyCode::Right => {
                debug!("Shift + → キーが押されました");
                // 大きくページを進める（10ページ）
                for _ in 0..10 {
                    PageManager::next_page(state);
                    if !PageManager::has_next_page(state) {
                        break;
                    }
                }
            }
            _ => {
                debug!("未定義のShift組み合わせ: {:?}", key_code);
            }
        }
    }

    /// Ctrl + キーの組み合わせ処理
    fn handle_ctrl_key_combination(
        state: &mut AppState,
        key_code: KeyCode
    ) {
        match key_code {
            KeyCode::R => {
                debug!("Ctrl + R キーが押されました");
                // アプリケーションのリセット
                state.reset();
                info!("アプリケーションがリセットされました");
            }
            KeyCode::Q => {
                debug!("Ctrl + Q キーが押されました");
                // 終了コマンド（実際の終了は上位で処理）
                info!("終了が要求されました");
            }
            _ => {
                debug!("未定義のCtrl組み合わせ: {:?}", key_code);
            }
        }
    }

    /// Alt + キーの組み合わせ処理
    fn handle_alt_key_combination(
        state: &mut AppState,
        key_code: KeyCode
    ) {
        match key_code {
            KeyCode::Enter => {
                debug!("Alt + Enter キーが押されました");
                // フルスクリーン切り替え（将来の実装用）
                info!("フルスクリーン切り替え（未実装）");
            }
            _ => {
                debug!("未定義のAlt組み合わせ: {:?}", key_code);
            }
        }
    }

    /// キーボードショートカット一覧を取得
    pub fn get_keyboard_shortcuts() -> Vec<KeyboardShortcut> {
        vec![
            KeyboardShortcut::new("←", "次のページ"),
            KeyboardShortcut::new("→", "前のページ"),
            KeyboardShortcut::new("↑", "前のファイル"),
            KeyboardShortcut::new("↓", "次のファイル"),
            KeyboardShortcut::new("1", "シングルページモード"),
            KeyboardShortcut::new("2", "ダブルページモード"),
            KeyboardShortcut::new("R", "回転モード切り替え"),
            KeyboardShortcut::new("Home", "最初のページ"),
            KeyboardShortcut::new("End", "最後のページ"),
            KeyboardShortcut::new("Page Up", "前のページ"),
            KeyboardShortcut::new("Page Down", "次のページ"),
            KeyboardShortcut::new("Space", "次のページ"),
            KeyboardShortcut::new("Backspace", "前のページ"),
            KeyboardShortcut::new("Shift + ←", "10ページ戻る"),
            KeyboardShortcut::new("Shift + →", "10ページ進む"),
            KeyboardShortcut::new("Ctrl + R", "リセット"),
        ]
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// キーボードショートカット情報
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub key: String,
    pub description: String,
}

impl KeyboardShortcut {
    pub fn new(key: &str, description: &str) -> Self {
        Self {
            key: key.to_string(),
            description: description.to_string(),
        }
    }
}
