use iced::{
    executor, Application, Command, Element, Settings, Subscription, Theme,
};

// Model-View-Controller のモジュールをインポート
mod model;
mod view;
mod controller;

// 既存のモジュールをインポート
mod reader_rar5;
mod reader_rar4;
mod reader_zip;
mod archive_reader;
mod file_checker;
mod sort_filename;
mod compress_deflate;

use model::app_state::AppState;
use view::app_view::AppView;
use controller::app_controller::{AppController, Message};

/// アプリケーションのメインエントリーポイント
pub fn main() -> iced::Result {
    // フォントを指定しつつ実行
    ImageViewerApp::run(Settings {
        //default_font: Some(include_bytes!("../fonts/NotoSansJP-Regular.ttf")),
        ..Settings::default()
    })
}

/// メインアプリケーション構造体
/// MVCパターンにより、この構造体は非常にシンプルになりました
#[derive(Debug, Default)]
struct ImageViewerApp {
    state: AppState,
}

impl Application for ImageViewerApp {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    /// アプリケーションの初期化
    fn new(_flags: ()) -> (ImageViewerApp, Command<Message>) {
        let (state, command) = AppController::initialize();
        
        (
            ImageViewerApp { state },
            command
        )
    }

    /// ウィンドウのタイトル
    fn title(&self) -> String {
        let base_title = "Saten - 画像ビューア";
        
        if self.state.has_files() {
            let page_info = format!(
                " - {} / {} ファイル", 
                self.state.current_file_index + 1, 
                self.state.total_files
            );
            format!("{}{}", base_title, page_info)
        } else {
            base_title.to_string()
        }
    }

    /// メッセージ処理の委譲
    /// 実際の処理はControllerに委譲することで、main.rsをシンプルに保つ
    fn update(&mut self, message: Message) -> Command<Message> {
        AppController::update(&mut self.state, message)
    }

    /// イベント購読の設定
    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::events().map(Message::EventOccurred)
    }

    /// ビューの構築
    /// 実際の描画処理はViewに委譲
    fn view(&self) -> Element<Message> {
        AppView::build(&self.state)
    }
}

/// アプリケーション終了時の処理
impl Drop for ImageViewerApp {
    fn drop(&mut self) {
        AppController::shutdown(&self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initialization() {
        let (app, _) = ImageViewerApp::new(());
        assert!(!app.state.has_files());
        assert_eq!(app.state.current_file_index, 0);
        assert_eq!(app.state.total_files, 0);
    }

    #[test]
    fn test_title_without_files() {
        let app = ImageViewerApp::default();
        assert_eq!(app.title(), "Saten - 画像ビューア");
    }
}
