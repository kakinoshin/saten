use iced::{
    Alignment, Element, Length,
};
use iced::widget::{
    Container, Text, Column, Row, Space,
};

use crate::controller::app_controller::Message;

pub struct LayoutHelper;

impl LayoutHelper {
    /// 中央揃えのコンテナを作成
    pub fn center_container<T>(
        content: T
    ) -> Container<'static, Message>
    where
        T: Into<Element<'static, Message>>,
    {
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
    }

    /// フルサイズのコンテナを作成
    pub fn full_size_container<T>(
        content: T
    ) -> Container<'static, Message>
    where
        T: Into<Element<'static, Message>>,
    {
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
    }

    /// パディング付きのコンテナを作成
    pub fn padded_container<T>(
        content: T,
        padding: u16
    ) -> Container<'static, Message>
    where
        T: Into<Element<'static, Message>>,
    {
        Container::new(content)
            .padding(padding)
    }

    /// 垂直レイアウトを作成
    pub fn vertical_layout() -> Column<'static, Message> {
        Column::new()
            .width(Length::Fill)
            .align_x(Alignment::Start)
            .spacing(4)
    }

    /// 水平レイアウトを作成
    pub fn horizontal_layout() -> Row<'static, Message> {
        Row::new()
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .spacing(4)
    }

    /// 見出しテキストを作成
    pub fn create_heading(text: &str, size: f32) -> Text<'static> {
        Text::new(text.to_string())  // 借用データを所有データに変換
            .size(size)
    }

    /// 本文テキストを作成
    pub fn create_body_text(text: &str) -> Text<'static> {
        Text::new(text.to_string())  // 借用データを所有データに変換
            .size(14)
    }

    /// スペーサーを作成
    pub fn create_spacer(size: u16) -> Space {
        Space::new().height(Length::Fixed(size as f32))
    }

    /// 分割線を作成
    pub fn create_divider() -> Container<'static, Message> {
        Container::new(Space::new().height(Length::Fixed(1.0)))
            .width(Length::Fill)
    }

    /// 情報パネルを作成
    pub fn create_info_panel<T>(
        title: &str,
        content: T
    ) -> Container<'static, Message>
    where
        T: Into<Element<'static, Message>>,
    {
        let title_text = Self::create_heading(title, 16.0);
        let panel_content = Self::vertical_layout()
            .push(title_text)
            .push(Self::create_spacer(8))
            .push(content);

        Container::new(panel_content)
            .padding(12)
    }

    /// エラーメッセージを作成
    pub fn create_error_message(message: &str) -> Text<'static> {
        Text::new(format!("エラー: {}", message))  // format!で所有データを作成
            .size(14)
            .color(iced::Color::from_rgb(0.8, 0.2, 0.2))
    }

    /// 成功メッセージを作成
    pub fn create_success_message(message: &str) -> Text<'static> {
        Text::new(message.to_string())  // 借用データを所有データに変換
            .size(14)
            .color(iced::Color::from_rgb(0.2, 0.8, 0.2))
    }

    /// 警告メッセージを作成
    pub fn create_warning_message(message: &str) -> Text<'static> {
        Text::new(format!("警告: {}", message))  // format!で所有データを作成
            .size(14)
            .color(iced::Color::from_rgb(0.8, 0.6, 0.2))
    }

    /// 2列レイアウトを作成
    pub fn create_two_column_layout<L, R>(
        left_content: L,
        right_content: R
    ) -> Row<'static, Message>
    where
        L: Into<Element<'static, Message>>,
        R: Into<Element<'static, Message>>,
    {
        Row::new()
            .width(Length::Fill)
            .spacing(8)
            .push(
                Container::new(left_content)
                    .width(Length::FillPortion(1))
            )
            .push(
                Container::new(right_content)
                    .width(Length::FillPortion(1))
            )
    }

    /// 3列レイアウトを作成
    pub fn create_three_column_layout<L, C, R>(
        left_content: L,
        center_content: C,
        right_content: R
    ) -> Row<'static, Message>
    where
        L: Into<Element<'static, Message>>,
        C: Into<Element<'static, Message>>,
        R: Into<Element<'static, Message>>,
    {
        Row::new()
            .width(Length::Fill)
            .spacing(8)
            .push(
                Container::new(left_content)
                    .width(Length::FillPortion(1))
            )
            .push(
                Container::new(center_content)
                    .width(Length::FillPortion(2))
            )
            .push(
                Container::new(right_content)
                    .width(Length::FillPortion(1))
            )
    }

    /// レスポンシブなコンテナを作成
    pub fn create_responsive_container<T>(
        content: T,
        max_width: u32
    ) -> Container<'static, Message>
    where
        T: Into<Element<'static, Message>>,
    {
        Container::new(content)
            .max_width(max_width)
            .align_x(Alignment::Center)
            .padding(16)
    }
}

// カスタムスタイルは将来の拡張用に予約
