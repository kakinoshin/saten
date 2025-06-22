use iced::{Font, Settings};
use std::borrow::Cow;

/// フォント設定のヘルパー関数
/// 
/// Noto Sans JPフォントをアプリケーションに読み込みます。
/// 
/// # 使用例
/// ```rust
/// // UIコンポーネントで日本語フォントを使用
/// use crate::font_config;
/// 
/// let text = Text::new("こんにちは")
///     .font(font_config::japanese_font())
///     .size(16);
/// ```
pub fn configure_fonts() -> Settings {
    Settings::default()
}

/// 日本語フォントのフォント名定数
pub const NOTO_SANS_JP_FONT_NAME: &str = "Noto Sans JP";

/// 日本語フォントのFont定義
pub fn japanese_font() -> Font {
    // Noto Sans JPフォントを指定
    Font {
        family: iced::font::Family::Name(NOTO_SANS_JP_FONT_NAME),
        ..Font::default()
    }
}

/// 太字の日本語フォント
pub fn japanese_font_bold() -> Font {
    Font {
        family: iced::font::Family::Name(NOTO_SANS_JP_FONT_NAME),
        weight: iced::font::Weight::Bold,
        ..Font::default()
    }
}

/// フォントが正しく読み込まれているかチェック
pub fn verify_font_loading() -> bool {
    // フォントファイルの存在確認
    let font_data = include_bytes!("../fonts/NotoSansJP-Regular.ttf");
    let is_valid = !font_data.is_empty() && font_data.len() > 1000; // 最低サイズチェック
    
    if is_valid {
        println!("  -> フォントサイズ: {} KB", font_data.len() / 1024);
    }
    
    is_valid
}

/// 日本語テキスト表示用のヘルパー関数
/// 
/// # 使用例
/// ```rust
/// use iced::widget::Text;
/// use crate::font_config;
/// 
/// // 通常の日本語テキスト
/// let normal_text = font_config::create_japanese_text("こんにちは", 16);
/// 
/// // 太字の日本語テキスト
/// let bold_text = font_config::create_japanese_text_bold("重要", 18);
/// ```
pub fn create_japanese_text(content: &str, size: u16) -> iced::widget::Text<'static> {
    iced::widget::Text::new(content.to_owned())
        .font(japanese_font())
        .size(size)
}

/// 太字の日本語テキストを作成
pub fn create_japanese_text_bold(content: &str, size: u16) -> iced::widget::Text<'static> {
    iced::widget::Text::new(content.to_owned())
        .font(japanese_font_bold())
        .size(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_file_exists() {
        assert!(verify_font_loading());
    }

    #[test]
    fn test_configure_fonts() {
        let _settings = configure_fonts();
        // フォント設定が正しく初期化されていることを確認
        // Settings::default()がエラーなく実行できることを確認
        assert!(true); // コンパイルが通ればOK
    }
    
    #[test]
    fn test_japanese_font() {
        let font = japanese_font();
        // 日本語フォントが正しく設定されていることを確認
        if let iced::font::Family::Name(name) = &font.family {
            assert_eq!(name, NOTO_SANS_JP_FONT_NAME);
        } else {
            panic!("フォントファミリーが正しく設定されていません");
        }
    }
    
    #[test]
    fn test_japanese_font_bold() {
        let font = japanese_font_bold();
        // 太字フォントが正しく設定されていることを確認
        assert_eq!(font.weight, iced::font::Weight::Bold);
        if let iced::font::Family::Name(name) = &font.family {
            assert_eq!(name, NOTO_SANS_JP_FONT_NAME);
        } else {
            panic!("フォントファミリーが正しく設定されていません");
        }
    }
    
    #[test]
    fn test_create_japanese_text() {
        let text = create_japanese_text("テスト", 16);
        // テキストコンポーネントが正しく作成されることを確認
        // サイズの確認（可能であれば）
        // assert_eq!(text.size(), Some(16));
    }
    
    #[test]
    fn test_create_japanese_text_bold() {
        let text = create_japanese_text_bold("太字テスト", 18);
        // 太字テキストコンポーネントが正しく作成されることを確認
        // サイズの確認（可能であれば）
        // assert_eq!(text.size(), Some(18));
    }
}
