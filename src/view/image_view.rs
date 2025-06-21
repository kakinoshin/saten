use iced::{
    Alignment, Element, Length,
    alignment,
};
use iced::widget::{
    Container, Row,
};

use log::{warn, error, debug};

use crate::model::app_state::AppState;
use crate::model::archive_manager::ArchiveManager;
use crate::model::image_manager::ImageManager;
use crate::controller::app_controller::Message;

pub struct ImageView;

impl ImageView {
    pub fn new() -> Self {
        Self
    }

    /// シングル画像表示を作成
    pub fn create_single_image(
        state: &AppState,
        file_index: usize
    ) -> Container<'static, Message> {
        let handle = Self::get_image_handle(state, file_index);

        Container::new(
            iced::widget::image::Viewer::new(handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
    }

    /// ダブル画像表示を作成
    pub fn create_double_image(state: &AppState) -> Row<'static, Message> {
        let handle_1 = Self::get_image_handle(state, state.current_file_index);
        let handle_2 = Self::get_image_handle(state, state.current_file_index + 1);

        // 回転モードに応じて画像の順序を変更
        let (left_handle, right_handle) = if state.rotate_mode {
            (handle_2, handle_1)
        } else {
            (handle_1, handle_2)
        };

        let image_left = Container::new(
            iced::widget::image::Viewer::new(left_handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Center);

        let image_right = Container::new(
            iced::widget::image::Viewer::new(right_handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Left)
        .align_y(alignment::Vertical::Center);

        Row::new()
            .width(Length::Fill)
            .align_items(Alignment::Start)
            .push(image_left)
            .push(image_right)
    }

    /// 画像ハンドルを取得
    fn get_image_handle(
        state: &AppState, 
        file_index: usize
    ) -> iced::widget::image::Handle {
        // インデックスの妥当性チェック
        if file_index >= state.archive_files.len() {
            warn!("無効なファイルインデックス: {} >= {}", file_index, state.archive_files.len());
            return ImageManager::create_error_image();
        }

        let file = &state.archive_files[file_index];
        debug!("描画中: {} (offset: {}, size: {}, fsize: {})", 
            file.filepath, file.offset, file.size, file.fsize);

        // ファイルデータを解凍
        let data = match ArchiveManager::decompress_file_data(&state.archive_buffer, file) {
            Ok(data) => data,
            Err(e) => {
                error!("ファイルの解凍に失敗: {}", e);
                return ImageManager::create_error_image();
            }
        };

        // 画像ハンドルを作成
        match ImageManager::create_image_handle(&data, state.rotate_mode) {
            Ok(handle) => handle,
            Err(e) => {
                error!("画像の作成に失敗: {}", e);
                ImageManager::create_error_image()
            }
        }
    }

    /// プレビュー画像を作成（サムネイル用）
    pub fn create_thumbnail_image(
        state: &AppState,
        file_index: usize,
        size: u16
    ) -> Container<'static, Message> {
        let handle = Self::get_image_handle(state, file_index);

        Container::new(
            iced::widget::image::Viewer::new(handle)
        )
        .width(Length::Fixed(size as f32))
        .height(Length::Fixed(size as f32))
        .padding(2)
    }

    /// 画像情報を取得してテキスト表示
    pub fn get_image_info_text(state: &AppState, file_index: usize) -> String {
        if let Some(file) = state.get_file(file_index) {
            format!(
                "ファイル: {}\nサイズ: {} bytes\n圧縮: {:?}",
                file.filename,
                file.fsize,
                file.ctype
            )
        } else {
            "画像情報が取得できません".to_string()
        }
    }

    /// 画像が有効かチェック
    pub fn is_valid_image(state: &AppState, file_index: usize) -> bool {
        if file_index >= state.archive_files.len() {
            return false;
        }

        let file = &state.archive_files[file_index];
        
        // ファイルサイズチェック
        if file.size == 0 || file.fsize == 0 {
            return false;
        }

        // 解凍を試行
        match ArchiveManager::decompress_file_data(&state.archive_buffer, file) {
            Ok(data) => ImageManager::validate_image_data(&data),
            Err(_) => false,
        }
    }

    /// 複数画像のグリッド表示を作成（ギャラリービュー用）
    pub fn create_grid_view(
        _state: &AppState,
        _columns: usize,
        _thumbnail_size: u16
    ) -> Container<'static, Message> {
        // 実装は必要に応じて追加
        Container::new(
            iced::widget::Text::new("グリッドビュー（未実装）")
        )
    }
}

impl Default for ImageView {
    fn default() -> Self {
        Self::new()
    }
}
