use iced::{
    Alignment, Element, Length,
    alignment,
};
use iced::widget::{
    Container, Text, Column, Row,
};

use crate::model::app_state::{AppState, DisplayMode};
use crate::view::image_view::ImageView;
use crate::view::layout::LayoutHelper;
use crate::controller::app_controller::Message;

pub struct AppView;

impl AppView {
    pub fn new() -> Self {
        Self
    }

    /// メインビューを構築
    pub fn build(state: &AppState) -> Element<'static, Message> {
        // ファイルパス表示部 - データを所有するようにクローン
        let path_display = Self::create_path_display(state);

        // 画像表示部
        let image_display = if state.has_files() {
            match state.display_mode {
                DisplayMode::Single => {
                    Self::create_single_view(state)
                }
                DisplayMode::Double => {
                    Self::create_double_view(state)
                }
            }
        } else {
            Self::create_empty_view()
        };

        // メインコンテンツを組み立て
        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Start)
            .push(path_display)
            .push(image_display);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// ファイルパス表示を作成
    fn create_path_display(state: &AppState) -> Container<'static, Message> {
        // 借用データを所有データに変換
        let path_text = if state.file_path_string().is_empty() {
            "画像ファイルをウィンドウにドロップしてね。".to_string()
        } else {
            state.file_path_string()  // これは既にStringを返す
        };

        Container::new(Text::new(path_text).size(20)).padding(4)
    }

    /// シングルページビューを作成
    fn create_single_view(state: &AppState) -> Container<'static, Message> {
        let image_container = ImageView::create_single_image(state, state.current_file_index);

        Container::new(image_container)
            .height(Length::Fill)
            .width(Length::Fill)
    }

    /// ダブルページビューを作成
    fn create_double_view(state: &AppState) -> Container<'static, Message> {
        let double_view = ImageView::create_double_image(state);

        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Start)
            .push(double_view);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
    }

    /// 空のビューを作成（ファイルが読み込まれていない場合）
    fn create_empty_view() -> Container<'static, Message> {
        Container::new(Text::new("empty").size(20)).padding(4)
    }

    /// ステータス情報を表示（オプション）
    pub fn create_status_bar(state: &AppState) -> Container<'static, Message> {
        // 借用データから所有データを作成
        let status_text = if state.has_files() {
            format!(
                "ページ: {} / {} | モード: {} | 回転: {}",
                state.current_file_index + 1,
                state.total_files,
                state.display_mode,  // {:?} ではなく {} を使用
                if state.rotate_mode { "ON" } else { "OFF" }
            )
        } else {
            "ファイルが読み込まれていません".to_string()
        };

        Container::new(
            Text::new(status_text)
                .size(14)
        ).padding(4)
    }

    /// エラー表示を作成
    pub fn create_error_view(error_message: &str) -> Element<'static, Message> {
        // エラーメッセージを所有データとして作成
        let error_text = Text::new(format!("エラー: {}", error_message))
            .size(18)
            .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.0, 0.0)));

        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(
                Container::new(error_text)
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill)
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// 読み込み中表示を作成
    pub fn create_loading_view() -> Element<'static, Message> {
        let loading_text = Text::new("読み込み中...")
            .size(18);

        let content = Column::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(
                Container::new(loading_text)
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill)
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Default for AppView {
    fn default() -> Self {
        Self::new()
    }
}
