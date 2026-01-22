use iced::widget::{button, column, container, text, row, Image};
use iced::{Element, Subscription, Task, Theme, Event, Length, Alignment, Size};
use iced::keyboard::{Event as KeyboardEvent, Key};
use log::debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::fs::File;
use clap::Parser;
use memmap2::Mmap;

// アーカイブ処理モジュールをインポート
mod archive_reader;
mod reader_zip;
mod reader_rar4;
mod reader_rar5;
mod file_checker;
mod sort_filename;
mod compress_deflate;
mod rar_handler;

use archive_reader::{ArcReader, MemberFile, CompressionType, ArchiveResult};
use reader_zip::ZipReader;
use reader_rar4::Rar4Reader;
use reader_rar5::Rar5Reader;

// IPC module for single instance support
mod ipc;

// MVC modules (for AppState used in ImageViewer)
mod model;
mod view;
mod controller;

use model::app_state::AppState;

/// Saten - Image Viewer for Archive Files
#[derive(Parser, Debug)]
#[command(name = "saten")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to archive file to open (.rar, .zip, .cbr, .cbz)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

/// アプリケーションのメインエントリーポイント
pub fn main() -> iced::Result {
    // Parse command line arguments
    let args = Args::parse();

    // Try to become primary instance or send to existing instance
    let (ipc_receiver, socket_path) = match ipc::try_become_primary(args.file.clone()) {
        ipc::InstanceResult::Primary { receiver, socket_path } => {
            (Some(receiver), Some(socket_path))
        }
        ipc::InstanceResult::Secondary => {
            // Another instance is running, we sent the file path, now exit
            println!("Another instance is already running. File path sent.");
            std::process::exit(0);
        }
    };

    // Wrap the receiver in Arc<Mutex<Option>> for sharing with iced subscription
    let ipc_receiver_shared: Arc<Mutex<Option<mpsc::Receiver<PathBuf>>>> =
        Arc::new(Mutex::new(ipc_receiver));

    // Create flags to pass initial file path and IPC receiver
    let flags = AppFlags {
        initial_file: args.file,
        ipc_receiver: ipc_receiver_shared,
        socket_path,
    };

    iced::application(
        move || {
            // Create ImageViewer with IPC receiver and socket path
            let viewer = ImageViewer {
                ipc_receiver: flags.ipc_receiver.clone(),
                socket_path: flags.socket_path.clone(),
                ..ImageViewer::default()
            };

            // If initial file is provided, trigger file load
            let task = if let Some(file_path) = flags.initial_file.clone() {
                Task::done(Message::FileDropped(file_path))
            } else {
                Task::none()
            };

            (viewer, task)
        },
        ImageViewer::update,
        ImageViewer::view,
    )
        .title(ImageViewer::title)
        .subscription(ImageViewer::subscription)
        .theme(ImageViewer::theme)
        .window_size(Size::new(1200.0, 800.0))
        .resizable(true)
        .run()
}

/// Application initialization flags
#[derive(Clone)]
pub struct AppFlags {
    pub initial_file: Option<PathBuf>,
    pub ipc_receiver: Arc<Mutex<Option<mpsc::Receiver<PathBuf>>>>,
    pub socket_path: Option<PathBuf>,
}

impl Default for AppFlags {
    fn default() -> Self {
        AppFlags {
            initial_file: None,
            ipc_receiver: Arc::new(Mutex::new(None)),
            socket_path: None,
        }
    }
}

impl std::fmt::Debug for AppFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppFlags")
            .field("initial_file", &self.initial_file)
            .field("socket_path", &self.socket_path)
            .finish()
    }
}


#[derive(Debug, Default, Clone)]
pub enum DisplayMode {
    Single,
    #[default]
    Double,
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayMode::Single => write!(f, "シングル"),
            DisplayMode::Double => write!(f, "ダブル"),
        }
    }
}

struct ImageViewer {
    // 基本状態
    current_file: Option<PathBuf>,
    status_message: String,

    // アーカイブ関連
    archive_files: Vec<MemberFile>,
    current_file_index: usize,
    total_files: usize,
    
    // 表示状態
    display_mode: DisplayMode,
    rotation_angle: u16,    // 画像回転角度 (0, 90, 180, 270)
    flip_mode: bool,        // フリップモード (左右入れ替え)
    fullsize_mode: bool,    // フルサイズ表示モード
    show_overlay: bool,     // オーバーレイ表示状態
    
    // 画像ハンドル
    image_handles: Vec<iced::widget::image::Handle>,
    
    // ウィンドウ情報
    window_size: Size,

    // アプリケーションの状態管理（二重起動防止）
    state: AppState,
    ipc_receiver: Arc<Mutex<Option<mpsc::Receiver<PathBuf>>>>,
    socket_path: Option<PathBuf>,
}

impl std::fmt::Debug for ImageViewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageViewer")
            .field("state", &self.state)
            .field("socket_path", &self.socket_path)
            .finish()
    }
}

#[derive(Debug, Clone)]
enum Message {
    FileDropped(PathBuf),
    ClearFile,
    EventOccurred(Event),
    ArchiveLoaded(Result<Vec<MemberFile>, String>),
    ImagesLoaded(Result<Vec<iced::widget::image::Handle>, String>),
    
    // ナビゲーション
    NextPage,
    PreviousPage,
    NextFile,
    PreviousFile,
    
    // 表示モード
    SetSingleMode,
    SetDoubleMode,
    ToggleRotate,   // 画像回転 (90度)
    ToggleFlip,     // 左右入れ替え
    ToggleFullsize,
    ToggleOverlay,  // オーバーレイ表示切り替え
    
    // ページジャンプ
    GotoFirstPage,
    GotoLastPage,
    
    // ウィンドウイベント
    WindowResized(Size),

    // IPC
    IpcFileReceived(PathBuf),
}

impl Default for ImageViewer {
    fn default() -> Self {
        ImageViewer {
            current_file: None,
            status_message: String::new(),
            archive_files: Vec::new(),
            current_file_index: 0,
            total_files: 0,
            display_mode: DisplayMode::default(),
            rotation_angle: 0,
            flip_mode: true,  // デフォルトでフリップモード（右から左へ読む）
            fullsize_mode: false,
            show_overlay: false,
            image_handles: Vec::new(),
            window_size: Size::new(1200.0, 800.0),
            state: AppState::default(),
            ipc_receiver: Arc::new(Mutex::new(None)),
            socket_path: None,
        }
    }
}

impl ImageViewer {
    fn title(&self) -> String {
        // 現在表示中の画像ファイル名を表示（ダブルビューでは1枚目）
        if let Some(file) = self.archive_files.get(self.current_file_index) {
            format!("Saten - {}", file.filename)
        } else {
            "Saten".to_string()
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileDropped(path) => {
                println!("ファイルがドロップされました: {:?}", path);
                
                if Self::is_archive_file(&path) {
                    self.current_file = Some(path.clone());
                    self.status_message = format!("アーカイブを読み込み中: {}", path.display());
                    return Task::perform(Self::load_archive(path), Message::ArchiveLoaded);
                } else if Self::is_image_file(&path) {
                    self.current_file = Some(path.clone());
                    self.status_message = format!("画像を読み込み中: {}", path.display());
                    // 単一画像の場合もアーカイブとして扱う
                    return Task::perform(Self::load_single_image_as_archive(path), Message::ArchiveLoaded);
                } else {
                    self.status_message = "サポートされていないファイル形式です".to_string();
                }
            }
            
            Message::ArchiveLoaded(result) => {
                match result {
                    Ok(files) => {
                        let image_files: Vec<MemberFile> = files
                            .into_iter()
                            .filter(|f| Self::is_image_filename(&f.filename))
                            .collect();

                        if image_files.is_empty() {
                            self.status_message = "画像ファイルが見つかりません".to_string();
                        } else {
                            self.archive_files = image_files;
                            self.total_files = self.archive_files.len();
                            self.current_file_index = 0;
                            self.status_message = format!("{}個の画像を読み込みました", self.total_files);

                            // 現在のページの画像を読み込み
                            return self.load_current_page_images();
                        }
                    }
                    Err(error) => {
                        self.status_message = format!("読み込みエラー: {}", error);
                    }
                }
            }
            
            Message::ImagesLoaded(result) => {
                match result {
                    Ok(handles) => {
                        self.image_handles = handles;
                        self.update_status_message();
                    }
                    Err(error) => {
                        self.status_message = format!("画像読み込みエラー: {}", error);
                    }
                }
            }
            
            // ナビゲーション
            Message::NextPage => {
                self.next_page();
                return self.load_current_page_images();
            }
            Message::PreviousPage => {
                self.previous_page();
                return self.load_current_page_images();
            }
            Message::NextFile => {
                self.next_file();
                return self.load_current_page_images();
            }
            Message::PreviousFile => {
                self.previous_file();
                return self.load_current_page_images();
            }
            
            // 表示モード
            Message::SetSingleMode => {
                self.display_mode = DisplayMode::Single;
                return self.load_current_page_images();
            }
            Message::SetDoubleMode => {
                self.display_mode = DisplayMode::Double;
                // ダブルページモードでは偶数インデックスから開始
                if self.current_file_index % 2 != 0 && self.current_file_index > 0 {
                    self.current_file_index -= 1;
                }
                return self.load_current_page_images();
            }
            Message::ToggleRotate => {
                // Cycle through: 0 -> 90 -> 180 -> 270 -> 0
                self.rotation_angle = (self.rotation_angle + 90) % 360;
                debug!("回転角度: {}°", self.rotation_angle);
                return self.load_current_page_images();
            }
            Message::ToggleFlip => {
                self.flip_mode = !self.flip_mode;
                debug!("フリップモード: {}", if self.flip_mode { "ON" } else { "OFF" });
            }
            Message::ToggleOverlay => {
                self.show_overlay = !self.show_overlay;
                debug!("オーバーレイ表示: {}", if self.show_overlay { "ON" } else { "OFF" });
            }
            Message::ToggleFullsize => {
                self.fullsize_mode = !self.fullsize_mode;
                debug!("フルサイズモード: {}", if self.fullsize_mode { "ON" } else { "OFF" });
            }
            
            // ページジャンプ
            Message::GotoFirstPage => {
                self.current_file_index = 0;
                return self.load_current_page_images();
            }
            Message::GotoLastPage => {
                if self.total_files > 0 {
                    match self.display_mode {
                        DisplayMode::Single => {
                            self.current_file_index = self.total_files - 1;
                        }
                        DisplayMode::Double => {
                            self.current_file_index = if self.total_files >= 2 {
                                if self.total_files % 2 == 0 {
                                    self.total_files - 2
                                } else {
                                    self.total_files - 1
                                }
                            } else {
                                0
                            };
                        }
                    }
                    return self.load_current_page_images();
                }
            }
            
            Message::ClearFile => {
                *self = Self::default();
                self.status_message = "ファイルをドロップしてください".to_string();
            }
            
            Message::WindowResized(size) => {
                self.window_size = size;
                debug!("ウィンドウサイズ変更: {}x{}", size.width, size.height);
            }
            
            Message::EventOccurred(event) => {
                // ウィンドウイベント処理
                if let Event::Window(iced::window::Event::FileDropped(path)) = event {
                    return self.update(Message::FileDropped(path));
                }

                if let Event::Window(iced::window::Event::Resized(size)) = event {
                    return self.update(Message::WindowResized(size));
                }

                // キーボードイベント処理
                if let Event::Keyboard(KeyboardEvent::KeyPressed { key, modifiers, .. }) = event {
                    return self.handle_keyboard_input(key, modifiers);
                }
            }

            Message::IpcFileReceived(path) => {
                // IPCから受信したファイルパスを処理
                debug!("IPCからファイルパスを受信: {:?}", path);
                return self.update(Message::FileDropped(path));
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // 画像表示エリア
        if !self.image_handles.is_empty() {
            if self.show_overlay {
                // オーバーレイ表示モード
                self.create_overlay_only_view()
            } else {
                // 画像のみ表示
                self.create_image_display_area()
            }
        } else {
            // 画像なしの場合のメッセージ
            let status_text = if self.status_message.is_empty() {
                "画像ファイルまたはアーカイブをドロップしてください\n\n[i] オーバーレイ表示"
            } else {
                &self.status_message
            };
            
            container(
                text(status_text)
                    .size(20)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        // Use iced 0.13 event listening API
        let event_sub = iced::event::listen().map(Message::EventOccurred);

        // IPC subscription for receiving file paths from secondary instances
        let ipc_sub = ipc::ipc_subscription(self.ipc_receiver.clone())
            .map(Message::IpcFileReceived);

        Subscription::batch([event_sub, ipc_sub])
    }

    // オーバーレイのみ表示を作成
    fn create_overlay_only_view(&self) -> Element<'_, Message> {
        let status_text = if self.status_message.is_empty() {
            "Saten - 画像ビューア"
        } else {
            &self.status_message
        };

        // 現在表示中の画像パス情報を取得
        let image_path_info = self.get_current_image_paths();

        // オーバーレイ内容
        let overlay_content = column![
            // ヘッダー情報
            text("Saten - 画像ビューア").size(24),
            text(status_text).size(16),
            text("").size(10), // スペーサー

            // 現在の画像パス情報
            text(image_path_info).size(12),
            text("").size(10), // スペーサー

            // コントロール
            self.create_overlay_controls(),

            // キーボードヘルプ
            text("[R] 回転 [F] 左右入替 [Z] フルサイズ [i] オーバーレイ [ESC] 終了").size(12),
        ]
        .spacing(10)
        .padding(20);

        // オーバーレイのみ表示
        container(overlay_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }

    // 現在表示中の画像パスを取得
    fn get_current_image_paths(&self) -> String {
        if self.archive_files.is_empty() {
            return "画像なし".to_string();
        }

        match self.display_mode {
            DisplayMode::Single => {
                if let Some(file) = self.archive_files.get(self.current_file_index) {
                    format!("📄 {}", file.filepath)
                } else {
                    "画像なし".to_string()
                }
            }
            DisplayMode::Double => {
                let mut paths = Vec::new();
                if let Some(file) = self.archive_files.get(self.current_file_index) {
                    paths.push(format!("📄 {}", file.filepath));
                }
                if let Some(file) = self.archive_files.get(self.current_file_index + 1) {
                    paths.push(format!("📄 {}", file.filepath));
                }
                paths.join("\n")
            }
        }
    }
    
    // オーバーレイ用のコントロールを作成
    fn create_overlay_controls(&self) -> Element<'_, Message> {
        let page_info = self.get_page_info_string();
        
        let nav_controls = row![
            button("<<").on_press(Message::GotoFirstPage),
            button("< 前").on_press_maybe(
                if self.has_previous_page() { Some(Message::PreviousPage) } else { None }
            ),
            text(page_info).size(14),
            button("次 >").on_press_maybe(
                if self.has_next_page() { Some(Message::NextPage) } else { None }
            ),
            button(">>").on_press(Message::GotoLastPage),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let mode_controls = row![
            button("シングル").on_press(Message::SetSingleMode),
            button("ダブル").on_press(Message::SetDoubleMode),
            button("回転").on_press(Message::ToggleRotate),
            button("左右入替").on_press(Message::ToggleFlip),
            button("フルサイズ").on_press(Message::ToggleFullsize),
            text(format!("{} {} {} {}",
                self.display_mode,
                if self.rotation_angle > 0 { format!("[{}°]", self.rotation_angle) } else { String::new() },
                if self.flip_mode { "[入替]" } else { "" },
                if self.fullsize_mode { "[フルサイズ]" } else { "" }
            )).size(12),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        column![
            nav_controls,
            mode_controls,
            button("クリア").on_press(Message::ClearFile),
        ]
        .spacing(10)
        .into()
    }

    // 画像表示エリアを作成
    fn create_image_display_area(&self) -> Element<'_, Message> {
        match self.display_mode {
            DisplayMode::Single => {
                if let Some(handle) = self.image_handles.first() {
                    container(
                        Image::new(handle.clone())
                            .width(Length::Fill)
                            .height(Length::Fill)
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
                } else {
                    container(text(""))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                }
            }
            DisplayMode::Double => {
                let mut row_content = row![].spacing(2);

                // 左ページ（フリップモードでは右ページ）
                if let Some(handle) = self.image_handles.get(if self.flip_mode { 1 } else { 0 }) {
                    row_content = row_content.push(
                        container(
                            Image::new(handle.clone())
                                .width(Length::Fill)
                                .height(Length::Fill)
                        )
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                    );
                } else {
                    row_content = row_content.push(
                        container(text(""))
                            .width(Length::FillPortion(1))
                            .height(Length::Fill)
                    );
                }

                // 右ページ（フリップモードでは左ページ）
                if let Some(handle) = self.image_handles.get(if self.flip_mode { 0 } else { 1 }) {
                    row_content = row_content.push(
                        container(
                            Image::new(handle.clone())
                                .width(Length::Fill)
                                .height(Length::Fill)
                        )
                        .width(Length::FillPortion(1))
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                    );
                } else {
                    row_content = row_content.push(
                        container(text(""))
                            .width(Length::FillPortion(1))
                            .height(Length::Fill)
                    );
                }

                container(row_content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }

    // キーボード入力を処理
    fn handle_keyboard_input(&mut self, key: Key, _modifiers: iced::keyboard::Modifiers) -> Task<Message> {
        match key.as_ref() {
            // ページナビゲーション
            Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                return Task::perform(async {}, |_| Message::NextPage);
            }
            Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                return Task::perform(async {}, |_| Message::PreviousPage);
            }
            Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                return Task::perform(async {}, |_| Message::PreviousFile);
            }
            Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                return Task::perform(async {}, |_| Message::NextFile);
            }
            
            // 表示モード切り替え
            Key::Character(ref c) if matches!(c.as_ref(), "1") => {
                return Task::perform(async {}, |_| Message::SetSingleMode);
            }
            Key::Character(ref c) if matches!(c.as_ref(), "2") => {
                return Task::perform(async {}, |_| Message::SetDoubleMode);
            }
            Key::Character(ref c) if matches!(c.as_ref(), "r" | "R") => {
                return Task::perform(async {}, |_| Message::ToggleRotate);
            }
            Key::Character(ref c) if matches!(c.as_ref(), "f" | "F") => {
                return Task::perform(async {}, |_| Message::ToggleFlip);
            }
            Key::Character(ref c) if matches!(c.as_ref(), "z" | "Z") => {
                return Task::perform(async {}, |_| Message::ToggleFullsize);
            }
            Key::Character(ref c) if matches!(c.as_ref(), "i" | "I") => {
                return Task::perform(async {}, |_| Message::ToggleOverlay);
            }
            
            // ページジャンプ
            Key::Named(iced::keyboard::key::Named::Home) => {
                return Task::perform(async {}, |_| Message::GotoFirstPage);
            }
            Key::Named(iced::keyboard::key::Named::End) => {
                return Task::perform(async {}, |_| Message::GotoLastPage);
            }
            Key::Named(iced::keyboard::key::Named::Space) => {
                return Task::perform(async {}, |_| Message::NextPage);
            }
            Key::Named(iced::keyboard::key::Named::Backspace) => {
                return Task::perform(async {}, |_| Message::PreviousPage);
            }
            Key::Named(iced::keyboard::key::Named::Escape) => {
                // ESCキーでオーバーレイまたはフルサイズモードを終了
                if self.show_overlay {
                    return Task::perform(async {}, |_| Message::ToggleOverlay);
                } else if self.fullsize_mode {
                    return Task::perform(async {}, |_| Message::ToggleFullsize);
                }
            }
            
            _ => {}
        }
        Task::none()
    }

    // === 以下、内部ロジック（既存のものをそのまま使用） ===

    // 現在のページの画像を読み込み
    fn load_current_page_images(&self) -> Task<Message> {
        if self.archive_files.is_empty() || self.current_file.is_none() {
            return Task::none();
        }

        let files_to_load = match self.display_mode {
            DisplayMode::Single => {
                if let Some(file) = self.archive_files.get(self.current_file_index) {
                    vec![file.clone()]
                } else {
                    vec![]
                }
            }
            DisplayMode::Double => {
                let mut files = vec![];
                if let Some(file) = self.archive_files.get(self.current_file_index) {
                    files.push(file.clone());
                }
                if let Some(file) = self.archive_files.get(self.current_file_index + 1) {
                    files.push(file.clone());
                }
                files
            }
        };

        if files_to_load.is_empty() {
            return Task::none();
        }

        let path = self.current_file.clone().unwrap();
        let rotation = self.rotation_angle;
        Task::perform(
            Self::load_images_from_archive(path, files_to_load, rotation),
            Message::ImagesLoaded
        )
    }

    // アーカイブから複数画像を読み込み (メモリマップ使用)
    async fn load_images_from_archive(
        path: PathBuf,
        files: Vec<MemberFile>,
        rotation_angle: u16,
    ) -> Result<Vec<iced::widget::image::Handle>, String> {
        // メモリマップでファイルを開く（必要な部分のみがメモリに読み込まれる）
        let file = File::open(&path)
            .map_err(|e| format!("ファイルオープンエラー: {}", e))?;
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| format!("メモリマップエラー: {}", e))?;

        let mut handles = Vec::new();

        for member_file in files {
            match Self::extract_single_image_from_archive(&mmap, &member_file, rotation_angle) {
                Ok(handle) => handles.push(handle),
                Err(e) => return Err(e),
            }
        }

        Ok(handles)
    }

    // GPU メモリ節約のための最大画像サイズ (ピクセル)
    const MAX_IMAGE_DIMENSION: u32 = 4096;

    // アーカイブから単一画像を抽出 (大きな画像はリサイズ)
    fn extract_single_image_from_archive(
        buffer: &[u8],
        file: &MemberFile,
        rotation_angle: u16,
    ) -> Result<iced::widget::image::Handle, String> {
        use image::io::Reader as ImageReader;
        use image::GenericImageView;
        use std::io::Cursor;

        // まず画像データを取得
        let image_data = match file.ctype {
            CompressionType::Uncompress => {
                let start = file.offset as usize;
                let end = start + file.size as usize;

                if end > buffer.len() {
                    return Err("ファイルサイズが範囲外です".to_string());
                }

                &buffer[start..end]
            }
            CompressionType::Deflate => {
                // Deflate の場合は一時バッファに展開
                return Self::extract_deflate_image(buffer, file, rotation_angle);
            }
            _ => {
                return Err(format!("未対応の圧縮形式: {:?}", file.ctype));
            }
        };

        // 画像をデコード
        let img = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()
            .map_err(|e| format!("画像フォーマット検出エラー: {}", e))?
            .decode()
            .map_err(|e| format!("画像デコードエラー: {}", e))?;

        // 回転処理
        let img = match rotation_angle {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            _ => img,
        };

        // 画像サイズをチェックしてリサイズ
        let (width, height) = img.dimensions();
        let img = if width > Self::MAX_IMAGE_DIMENSION || height > Self::MAX_IMAGE_DIMENSION {
            debug!("画像リサイズ: {}x{} -> max {}", width, height, Self::MAX_IMAGE_DIMENSION);
            img.resize(
                Self::MAX_IMAGE_DIMENSION,
                Self::MAX_IMAGE_DIMENSION,
                image::imageops::FilterType::Triangle,
            )
        } else {
            img
        };

        // JPEG としてエンコード (PNG より軽量)
        let mut output = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        img.write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|e| format!("画像エンコードエラー: {}", e))?;

        Ok(iced::widget::image::Handle::from_bytes(output))
    }

    // Deflate 圧縮された画像を抽出
    fn extract_deflate_image(
        buffer: &[u8],
        file: &MemberFile,
        rotation_angle: u16,
    ) -> Result<iced::widget::image::Handle, String> {
        use image::io::Reader as ImageReader;
        use image::GenericImageView;
        use std::io::Cursor;

        let decompressed = Self::decompress_deflate(buffer, file.offset, file.size)?;

        // 画像をデコード
        let img = ImageReader::new(Cursor::new(&decompressed))
            .with_guessed_format()
            .map_err(|e| format!("画像フォーマット検出エラー: {}", e))?
            .decode()
            .map_err(|e| format!("画像デコードエラー: {}", e))?;

        // 回転処理
        let img = match rotation_angle {
            90 => img.rotate90(),
            180 => img.rotate180(),
            270 => img.rotate270(),
            _ => img,
        };

        // 画像サイズをチェックしてリサイズ
        let (width, height) = img.dimensions();
        let img = if width > Self::MAX_IMAGE_DIMENSION || height > Self::MAX_IMAGE_DIMENSION {
            debug!("画像リサイズ: {}x{} -> max {}", width, height, Self::MAX_IMAGE_DIMENSION);
            img.resize(
                Self::MAX_IMAGE_DIMENSION,
                Self::MAX_IMAGE_DIMENSION,
                image::imageops::FilterType::Triangle,
            )
        } else {
            img
        };

        // JPEG としてエンコード
        let mut output = Vec::new();
        let mut cursor = Cursor::new(&mut output);
        img.write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|e| format!("画像エンコードエラー: {}", e))?;

        Ok(iced::widget::image::Handle::from_bytes(output))
    }

    // ページナビゲーション
    fn next_page(&mut self) {
        match self.display_mode {
            DisplayMode::Single => {
                if self.current_file_index + 1 < self.total_files {
                    self.current_file_index += 1;
                }
            }
            DisplayMode::Double => {
                if self.current_file_index + 2 < self.total_files {
                    self.current_file_index += 2;
                }
            }
        }
    }

    fn previous_page(&mut self) {
        match self.display_mode {
            DisplayMode::Single => {
                if self.current_file_index > 0 {
                    self.current_file_index -= 1;
                }
            }
            DisplayMode::Double => {
                if self.current_file_index >= 2 {
                    self.current_file_index -= 2;
                }
            }
        }
    }

    fn next_file(&mut self) {
        if self.current_file_index + 1 < self.total_files {
            self.current_file_index += 1;
        }
    }

    fn previous_file(&mut self) {
        if self.current_file_index > 0 {
            self.current_file_index -= 1;
        }
    }

    fn has_next_page(&self) -> bool {
        match self.display_mode {
            DisplayMode::Single => self.current_file_index + 1 < self.total_files,
            DisplayMode::Double => self.current_file_index + 2 < self.total_files,
        }
    }

    fn has_previous_page(&self) -> bool {
        match self.display_mode {
            DisplayMode::Single => self.current_file_index > 0,
            DisplayMode::Double => self.current_file_index >= 2,
        }
    }

    fn get_page_info_string(&self) -> String {
        if self.total_files == 0 {
            return "0 / 0".to_string();
        }
        
        match self.display_mode {
            DisplayMode::Single => {
                format!("{} / {}", self.current_file_index + 1, self.total_files)
            }
            DisplayMode::Double => {
                let end_index = (self.current_file_index + 2).min(self.total_files);
                if end_index > self.current_file_index + 1 {
                    format!("{}-{} / {}", 
                        self.current_file_index + 1, 
                        end_index, 
                        self.total_files)
                } else {
                    format!("{} / {}", self.current_file_index + 1, self.total_files)
                }
            }
        }
    }

    fn update_status_message(&mut self) {
        if !self.archive_files.is_empty() {
            self.status_message = format!(
                "{} | {} {}",
                self.get_page_info_string(),
                self.display_mode,
                if self.fullsize_mode { " | フルサイズ" } else { "" }
            );
        }
    }

    // === 以下、ユーティリティ関数（既存のものをそのまま使用） ===

    // Deflate圧縮を展開する
    fn decompress_deflate(
        buffer: &[u8],
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, String> {
        use std::io::Read;
        use flate2::read::DeflateDecoder;

        let start = offset as usize;
        let end = start + size as usize;
        
        if end > buffer.len() {
            return Err("範囲外アクセス".to_string());
        }

        let compressed_data = &buffer[start..end];
        let mut decoder = DeflateDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| format!("展開エラー: {}", e))?;
        
        Ok(decompressed)
    }

    // アーカイブファイルを読み込む (メモリマップ使用でメタデータのみ解析)
    async fn load_archive(path: PathBuf) -> Result<Vec<MemberFile>, String> {
        // メモリマップでファイルを開く（ヘッダー解析に必要な部分のみメモリに読み込まれる）
        let file = File::open(&path)
            .map_err(|e| format!("ファイルオープンエラー: {}", e))?;
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| format!("メモリマップエラー: {}", e))?;

        let mut files = Vec::new();

        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                let result = match ext_lower.as_str() {
                    "zip" | "cbz" => ZipReader::read_archive(&mmap, &mut files),
                    "rar" | "cbr" => {
                        if Self::is_rar5(&mmap) {
                            Rar5Reader::read_archive(&mmap, &mut files)
                        } else {
                            Rar4Reader::read_archive(&mmap, &mut files)
                        }
                    }
                    _ => return Err("サポートされていないアーカイブ形式".to_string()),
                };

                result.map_err(|e| format!("アーカイブ解析エラー: {}", e))?;
            }
        }

        if files.is_empty() {
            return Err("アーカイブ内にファイルが見つかりません".to_string());
        }

        Ok(files)
    }

    // 単一画像をアーカイブとして読み込む (メタデータのみ取得)
    async fn load_single_image_as_archive(path: PathBuf) -> Result<Vec<MemberFile>, String> {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("ファイルメタデータ取得エラー: {}", e))?;

        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image")
            .to_string();

        let member_file = MemberFile {
            filepath: filename.clone(),
            filename,
            offset: 0,
            size: metadata.len(),
            fsize: metadata.len(),
            ctype: CompressionType::Uncompress,
        };

        Ok(vec![member_file])
    }

    // RAR5かどうかを判定
    fn is_rar5(buffer: &[u8]) -> bool {
        const RAR5_SIGNATURE: &[u8] = b"Rar!\x1a\x07\x01\x00";
        buffer.len() >= RAR5_SIGNATURE.len() && 
        buffer.windows(RAR5_SIGNATURE.len())
            .any(|window| window == RAR5_SIGNATURE)
    }

    // アーカイブファイルかどうかをチェック
    fn is_archive_file(path: &PathBuf) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return matches!(
                    ext_lower.as_str(),
                    "zip" | "rar" | "cbz" | "cbr"
                );
            }
        }
        false
    }

    // 画像ファイルかどうかをチェック
    fn is_image_file(path: &PathBuf) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return matches!(
                    ext_lower.as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif"
                );
            }
        }
        false
    }

    // ファイル名から画像ファイルかどうかをチェック
    fn is_image_filename(filename: &str) -> bool {
        let filename_lower = filename.to_lowercase();
        matches!(
            filename_lower.split('.').last().unwrap_or(""),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif"
        )
    }
}
