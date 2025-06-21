//! Saten画像ビューア - MVCアーキテクチャテスト
//! 
//! このテストファイルは、リファクタリング後の構造が
//! 正常に動作することを確認するためのものです。

use std::path::PathBuf;

// Model-View-Controller のモジュールをインポート
pub mod model;
pub mod view;
pub mod controller;

// 既存のモジュールをインポート
pub mod reader_rar5;
pub mod reader_rar4;
pub mod reader_zip;
pub mod archive_reader;
pub mod file_checker;
pub mod sort_filename;
pub mod compress_deflate;
pub mod rar_handler;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::app_state::{AppState, DisplayMode};
    use crate::model::archive_manager::ArchiveManager;
    use crate::model::image_manager::ImageManager;
    use crate::model::page_manager::PageManager;
    
    // Model層のテスト
    mod model_tests {
        use super::*;
        
        #[test]
        fn test_app_state_initialization() {
            let state = AppState::new();
            assert_eq!(state.current_file_index, 0);
            assert_eq!(state.total_files, 0);
            assert!(!state.has_files());
            assert!(!state.rotate_mode);
        }
        
        #[test]
        fn test_app_state_navigation() {
            let mut state = AppState::new();
            
            // ファイルを追加した状態をシミュレート
            state.total_files = 10;
            state.current_file_index = 0;
            
            // 次のページテスト
            state.next_page();
            assert_eq!(state.current_file_index, 2); // ダブルページモードでは2つ進む
            
            // 前のページテスト
            state.previous_page();
            assert_eq!(state.current_file_index, 0);
        }
        
        #[test]
        fn test_display_mode_switching() {
            let mut state = AppState::new();
            
            // 初期状態はダブルページ
            assert!(matches!(state.display_mode, DisplayMode::Double));
            
            // シングルページに切り替え
            state.set_display_mode(DisplayMode::Single);
            assert!(matches!(state.display_mode, DisplayMode::Single));
        }
        
        #[test]
        fn test_rotate_mode_toggle() {
            let mut state = AppState::new();
            
            // 初期状態は回転なし
            assert!(!state.rotate_mode);
            
            // 回転モード切り替え
            state.toggle_rotate_mode();
            assert!(state.rotate_mode);
            
            state.toggle_rotate_mode();
            assert!(!state.rotate_mode);
        }
        
        #[test]
        fn test_archive_manager_validation() {
            // ダミーファイル情報でテスト
            use crate::archive_reader::{MemberFile, CompressionType};
            
            let file = MemberFile {
                filepath: "test.jpg".to_string(),
                filename: "test.jpg".to_string(),
                offset: 0,
                size: 1024,
                fsize: 1024,
                ctype: CompressionType::Uncompress,
            };
            
            assert!(ArchiveManager::validate_file_info(&file));
            assert!(ArchiveManager::is_supported_compression(&file.ctype));
        }
        
        #[test]
        fn test_image_format_detection() {
            // JPEG シグネチャテスト
            let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
            assert!(ImageManager::validate_image_data(&jpeg_data));
            
            // PNG シグネチャテスト
            let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
            assert!(ImageManager::validate_image_data(&png_data));
            
            // 無効なデータテスト
            let invalid_data = vec![0x00, 0x00, 0x00, 0x00];
            assert!(!ImageManager::validate_image_data(&invalid_data));
        }
        
        #[test]
        fn test_page_manager_navigation() {
            let mut state = AppState::new();
            state.total_files = 10;
            
            // 次のページ移動テスト
            PageManager::next_page(&mut state);
            assert_eq!(state.current_file_index, 2);
            
            // 前のページ移動テスト
            PageManager::previous_page(&mut state);
            assert_eq!(state.current_file_index, 0);
            
            // 最初のページに移動
            state.current_file_index = 5;
            PageManager::goto_first_page(&mut state);
            assert_eq!(state.current_file_index, 0);
            
            // 最後のページに移動
            PageManager::goto_last_page(&mut state);
            assert!(state.current_file_index >= state.total_files - 2);
        }
    }
    
    // Controller層のテスト
    mod controller_tests {
        use super::*;
        use crate::controller::keyboard_handler::KeyboardHandler;
        
        #[test]
        fn test_keyboard_shortcuts_list() {
            let shortcuts = KeyboardHandler::get_keyboard_shortcuts();
            assert!(!shortcuts.is_empty());
            
            // 基本的なショートカットが含まれているかチェック
            let has_left_arrow = shortcuts.iter()
                .any(|s| s.key == "←");
            assert!(has_left_arrow);
        }
        
        #[test]
        fn test_file_extension_validation() {
            let rar_path = PathBuf::from("test.rar");
            let zip_path = PathBuf::from("test.zip");
            let txt_path = PathBuf::from("test.txt");
            
            // 拡張子が存在することを確認
            assert!(rar_path.extension().is_some());
            assert!(zip_path.extension().is_some());
            assert!(txt_path.extension().is_some());
        }
    }
    
    // 統合テスト
    mod integration_tests {
        use super::*;
        
        #[test]
        fn test_mvc_integration() {
            // AppStateの初期化
            let mut state = AppState::new();
            
            // ファイルパスの設定
            let test_path = PathBuf::from("test.rar");
            state.set_file_path(test_path);
            
            // パスが正しく設定されているかチェック
            assert!(!state.file_path_string().is_empty());
            
            // 状態のリセット
            state.reset();
            assert!(!state.has_files());
            assert_eq!(state.current_file_index, 0);
        }
        
        #[test]
        fn test_error_handling() {
            use crate::archive_reader::ArchiveError;
            
            // エラー生成のテスト
            let error = ArchiveError::UnsupportedFormat;
            assert!(!error.to_string().is_empty());
            
            let bounds_error = ArchiveError::OutOfBounds {
                offset: 100,
                size: 50,
                buffer_len: 120,
            };
            assert!(bounds_error.to_string().contains("範囲外"));
        }
    }
}

// ベンチマークテスト（cargo bench で実行）
#[cfg(feature = "benchmark")]
mod benches {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn benchmark_state_operations() {
        let mut state = AppState::new();
        state.total_files = 1000;
        
        let start = Instant::now();
        
        // 100回のページ移動操作
        for _ in 0..100 {
            state.next_page();
            if state.current_file_index >= 990 {
                state.reset();
                state.total_files = 1000;
            }
        }
        
        let duration = start.elapsed();
        println!("100回のページ移動操作: {:?}", duration);
        
        // パフォーマンスが妥当な範囲内であることを確認
        assert!(duration.as_millis() < 10); // 10ms以内
    }
}
