use log::{debug, info};
use iced::keyboard::{Event as KeyboardEvent, Key};

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
            KeyboardEvent::KeyPressed { key, modifiers, .. } => {
                Self::handle_key_press(state, key, modifiers);
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
        key: Key,
        _modifiers: iced::keyboard::Modifiers
    ) {
        match key.as_ref() {
            // ページナビゲーション
            Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                debug!("← キーが押されました");
                PageManager::next_page(state);
            }
            Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                debug!("→ キーが押されました");
                PageManager::previous_page(state);
            }
            
            // ファイルナビゲーション（上下キー）
            Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                debug!("↑ キーが押されました");
                PageManager::previous_file(state);
            }
            Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                debug!("↓ キーが押されました");
                PageManager::next_file(state);
            }

            // 表示モード切り替え
            Key::Character(ref c) if matches!(c.as_ref(), "1") => {
                debug!("1 キーが押されました");
                PageManager::set_display_mode(state, DisplayMode::Single);
            }
            Key::Character(ref c) if matches!(c.as_ref(), "2") => {
                debug!("2 キーが押されました");
                PageManager::set_display_mode(state, DisplayMode::Double);
            }

            // 回転モード切り替え
            Key::Character(ref c) if matches!(c.as_ref(), "r" | "R") => {
                debug!("R キーが押されました");
                PageManager::toggle_rotate_mode(state);
            }

            // ページジャンプ
            Key::Named(iced::keyboard::key::Named::Home) => {
                debug!("Home キーが押されました");
                PageManager::goto_first_page(state);
            }
            Key::Named(iced::keyboard::key::Named::End) => {
                debug!("End キーが押されました");
                PageManager::goto_last_page(state);
            }

            // ページ送り（Page Up/Down）
            Key::Named(iced::keyboard::key::Named::PageUp) => {
                debug!("Page Up キーが押されました");
                PageManager::previous_page(state);
            }
            Key::Named(iced::keyboard::key::Named::PageDown) => {
                debug!("Page Down キーが押されました");
                PageManager::next_page(state);
            }

            // スペースキー（ページ送り）
            Key::Named(iced::keyboard::key::Named::Space) => {
                debug!("Space キーが押されました");
                PageManager::next_page(state);
            }

            // Backspace（戻る）
            Key::Named(iced::keyboard::key::Named::Backspace) => {
                debug!("Backspace キーが押されました");
                PageManager::previous_page(state);
            }

            // その他のキー
            _ => {
                // 未定義のキーは無視
                debug!("未定義のキーが押されました: {:?}", key);
            }
        }
    }

    /// 修飾キーを考慮したキー処理
    pub fn handle_key_with_modifiers(
        state: &mut AppState,
        key: Key,
        modifiers: iced::keyboard::Modifiers
    ) {
        if modifiers.shift() {
            Self::handle_shift_key_combination(state, key);
        } else if modifiers.control() {
            Self::handle_ctrl_key_combination(state, key);
        } else if modifiers.alt() {
            Self::handle_alt_key_combination(state, key);
        } else {
            Self::handle_key_press(state, key, modifiers);
        }
    }

    /// Shift + キーの組み合わせ処理
    fn handle_shift_key_combination(
        state: &mut AppState,
        key: Key
    ) {
        match key.as_ref() {
            Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                debug!("Shift + ← キーが押されました");
                // 大きくページを戻る（10ページ）
                for _ in 0..10 {
                    PageManager::previous_page(state);
                    if !PageManager::has_previous_page(state) {
                        break;
                    }
                }
            }
            Key::Named(iced::keyboard::key::Named::ArrowRight) => {
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
                debug!("未定義のShift組み合わせ: {:?}", key);
            }
        }
    }

    /// Ctrl + キーの組み合わせ処理
    fn handle_ctrl_key_combination(
        state: &mut AppState,
        key: Key
    ) {
        match key.as_ref() {
            Key::Character(ref c) if matches!(c.as_ref(), "r" | "R") => {
                debug!("Ctrl + R キーが押されました");
                // アプリケーションのリセット
                state.reset();
                info!("アプリケーションがリセットされました");
            }
            Key::Character(ref c) if matches!(c.as_ref(), "q" | "Q") => {
                debug!("Ctrl + Q キーが押されました");
                // 終了コマンド（実際の終了は上位で処理）
                info!("終了が要求されました");
            }
            _ => {
                debug!("未定義のCtrl組み合わせ: {:?}", key);
            }
        }
    }

    /// Alt + キーの組み合わせ処理
    fn handle_alt_key_combination(
        _state: &mut AppState,
        key: Key
    ) {
        match key.as_ref() {
            Key::Named(iced::keyboard::key::Named::Enter) => {
                debug!("Alt + Enter キーが押されました");
                // フルスクリーン切り替え（将来の実装用）
                info!("フルスクリーン切り替え（未実装）");
            }
            _ => {
                debug!("未定義のAlt組み合わせ: {:?}", key);
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
