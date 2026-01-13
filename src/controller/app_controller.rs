use std::path::PathBuf;
use log::{info, warn, error, debug};
use iced::Task;

use crate::model::app_state::{AppState, DisplayMode};
use crate::model::page_manager::PageManager;
use crate::controller::keyboard_handler::KeyboardHandler;
use crate::controller::file_handler::FileHandler;

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(iced::event::Event),
    FileLoaded(Result<(Vec<u8>, Vec<crate::archive_reader::MemberFile>), String>),
    ShowError(String),
    ShowSuccess(String),
    /// Received file path from IPC (secondary instance)
    IpcFileReceived(PathBuf),
}

pub struct AppController;

impl AppController {
    pub fn new() -> Self {
        Self
    }

    /// メインのメッセージ処理
    pub fn update(
        state: &mut AppState,
        message: Message
    ) -> Task<Message> {
        match message {
            Message::EventOccurred(event) => {
                Self::handle_event(state, event)
            }
            Message::FileLoaded(result) => {
                Self::handle_file_loaded(state, result)
            }
            Message::ShowError(message) => {
                error!("エラー: {}", message);
                Task::none()
            }
            Message::ShowSuccess(message) => {
                info!("成功: {}", message);
                Task::none()
            }
            Message::IpcFileReceived(path) => {
                info!("IPCからファイルパスを受信: {:?}", path);
                FileHandler::handle_file_drop(state, path)
            }
        }
    }

    /// イベント処理の振り分け
    fn handle_event(
        state: &mut AppState,
        event: iced::event::Event
    ) -> Task<Message> {
        match event {
            iced::event::Event::Window(window_event) => {
                Self::handle_window_event(state, window_event)
            }
            iced::event::Event::Keyboard(keyboard_event) => {
                KeyboardHandler::handle_keyboard_event(state, keyboard_event);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    /// ウィンドウイベントの処理
    fn handle_window_event(
        state: &mut AppState,
        window_event: iced::window::Event
    ) -> Task<Message> {
        match window_event {
            iced::window::Event::FileDropped(path) => {
                FileHandler::handle_file_drop(state, path)
            }
            _ => Task::none(),
        }
    }

    /// ファイル読み込み完了の処理
    fn handle_file_loaded(
        state: &mut AppState,
        result: Result<(Vec<u8>, Vec<crate::archive_reader::MemberFile>), String>
    ) -> Task<Message> {
        match result {
            Ok((buffer, files)) => {
                state.set_archive_buffer(buffer);
                state.set_archive_files(files);
                info!("ファイルの読み込みが完了しました: {} 個のファイル", state.total_files);
                Task::none()
            }
            Err(error_message) => {
                error!("ファイルの読み込みに失敗: {}", error_message);
                state.reset();
                Task::none()
            }
        }
    }

    /// 表示モードの変更
    pub fn set_display_mode(state: &mut AppState, mode: DisplayMode) {
        PageManager::set_display_mode(state, mode);
    }

    /// 回転モードの切り替え
    pub fn toggle_rotate_mode(state: &mut AppState) {
        PageManager::toggle_rotate_mode(state);
    }

    /// ページナビゲーション - 次のページ
    pub fn next_page(state: &mut AppState) {
        PageManager::next_page(state);
    }

    /// ページナビゲーション - 前のページ
    pub fn previous_page(state: &mut AppState) {
        PageManager::previous_page(state);
    }

    /// ファイルナビゲーション - 次のファイル
    pub fn next_file(state: &mut AppState) {
        PageManager::next_file(state);
    }

    /// ファイルナビゲーション - 前のファイル
    pub fn previous_file(state: &mut AppState) {
        PageManager::previous_file(state);
    }

    /// 最初のページに移動
    pub fn goto_first_page(state: &mut AppState) {
        PageManager::goto_first_page(state);
    }

    /// 最後のページに移動
    pub fn goto_last_page(state: &mut AppState) {
        PageManager::goto_last_page(state);
    }

    /// 指定ページに移動
    pub fn goto_page(state: &mut AppState, page: usize) {
        PageManager::goto_page(state, page);
    }

    /// エラー処理
    pub fn handle_error(error: &str) -> Task<Message> {
        error!("エラーが発生しました: {}", error);
        // 借用データを所有データに変換してからasync moveに渡す
        let error_message = error.to_string();
        Task::perform(
            async move { error_message },
            Message::ShowError
        )
    }

    /// 成功メッセージの処理
    pub fn handle_success(message: &str) -> Task<Message> {
        info!("処理が成功しました: {}", message);
        // 借用データを所有データに変換してからasync moveに渡す
        let success_message = message.to_string();
        Task::perform(
            async move { success_message },
            Message::ShowSuccess
        )
    }

    /// エラー処理（String版）- パフォーマンスを重視する場合
    pub fn handle_error_owned(error: String) -> Task<Message> {
        error!("エラーが発生しました: {}", error);
        Task::perform(
            async move { error },
            Message::ShowError
        )
    }

    /// 成功メッセージの処理（String版）- パフォーマンスを重視する場合
    pub fn handle_success_owned(message: String) -> Task<Message> {
        info!("処理が成功しました: {}", message);
        Task::perform(
            async move { message },
            Message::ShowSuccess
        )
    }

    /// エラーを直接Messageに変換
    pub fn create_error_message(error: &str) -> Message {
        error!("エラーが発生しました: {}", error);
        Message::ShowError(error.to_string())
    }

    /// 成功メッセージを直接Messageに変換
    pub fn create_success_message(message: &str) -> Message {
        info!("処理が成功しました: {}", message);
        Message::ShowSuccess(message.to_string())
    }

    /// アプリケーションの初期化
    pub fn initialize() -> (AppState, Task<Message>) {
        let state = AppState::new();
        let command = Task::none();
        (state, command)
    }

    /// アプリケーションのリセット
    pub fn reset_application(state: &mut AppState) {
        state.reset();
        info!("アプリケーションがリセットされました");
    }

    /// 設定の保存（将来の拡張用）
    pub fn save_settings(_state: &AppState) -> Task<Message> {
        debug!("設定を保存します（未実装）");
        // 将来的にはファイルに設定を保存する処理を追加
        Task::none()
    }

    /// 設定の読み込み（将来の拡張用）
    pub fn load_settings() -> Task<Message> {
        debug!("設定を読み込みます（未実装）");
        // 将来的にはファイルから設定を読み込む処理を追加
        Task::none()
    }

    /// アプリケーションの終了処理
    pub fn shutdown(_state: &AppState) {
        info!("アプリケーションを終了します");
        // 必要に応じてクリーンアップ処理を追加
    }

    /// エラーハンドリングのヘルパー関数
    pub fn handle_result<T, E: std::fmt::Display>(
        result: Result<T, E>, 
        success_msg: &str
    ) -> Task<Message> {
        match result {
            Ok(_) => Self::handle_success(success_msg),
            Err(error) => Self::handle_error(&error.to_string()),
        }
    }

    /// 複数のエラーを統合してハンドリング
    pub fn handle_multiple_errors(errors: Vec<String>) -> Task<Message> {
        if errors.is_empty() {
            return Task::none();
        }

        let combined_error = if errors.len() == 1 {
            errors.into_iter().next().unwrap()
        } else {
            format!("複数のエラーが発生しました:\n{}", errors.join("\n"))
        };

        Self::handle_error_owned(combined_error)
    }

    /// 警告メッセージの処理
    pub fn handle_warning(warning: &str) -> Task<Message> {
        warn!("警告: {}", warning);
        // 警告は現在ログのみで、UIには表示しない
        // 将来的にはMessage::ShowWarningを追加することも可能
        Task::none()
    }
}

impl Default for AppController {
    fn default() -> Self {
        Self::new()
    }
}
