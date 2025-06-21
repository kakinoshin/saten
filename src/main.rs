use iced::{
    alignment, executor,
};
use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    Theme,
};
use iced::widget::{
    Container, Text, Column, Row, Image, 
};

use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::time::Instant;
use log::{info, warn, error, debug};

mod reader_rar5;
mod reader_rar4;
mod reader_zip;
mod archive_reader;
mod file_checker;
mod sort_filename;
mod compress_deflate;

use crate::reader_rar5::Rar5Reader;
use crate::reader_rar4::Rar4Reader;
use crate::reader_zip::ZipReader;
use crate::archive_reader::{ArcReader, ArchiveError, ArchiveResult};
use crate::archive_reader::{MemberFile, CompressionType};
use crate::file_checker::{FileType, check_file_type};
use crate::sort_filename::sort_filename;

use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage, DynamicImage};


pub fn main() -> iced::Result {
    // フォントを指定しつつ実行する。
    Events::run(Settings {
        //default_font: Some(include_bytes!("../fonts/NotoSansJP-Regular.ttf")),
        ..Settings::default()
    })
}

#[derive(Debug, Default)]
enum Pages {
    Single,
#[default] Double,
}
// メインとなる構造体。アプリで保持する状態を変数にする。
#[derive(Debug, Default)]
struct Events {
    path   : PathBuf,
    files  : Vec<MemberFile>,
    f_idx  : usize,
    f_max  : usize,
    buf    : Vec<u8>,
    page   : Pages,
    rotate : bool,
}

// 何らかの変更があったときに飛ぶメッセージ。今回はイベント発生のみ。
#[derive(Debug, Clone)]
enum Message {
    EventOccurred(iced::event::Event),
}

impl Application for Events {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Events, Command<Message>) {
        (Events::default(), Command::none())
    }

    // ウィンドウのタイトル。状態に合わせた動的な生成も可。
    fn title(&self) -> String {
        String::from("Image Viewer")
    }

    // 何らかの変更があったときに呼び出される。
    // 発生した事柄はenum（今回の場合はMessage）として伝えられる。
    // Icedのバージョン0.3から引数にClipboardが増えたが、使わないので無視。
    fn update(&mut self, message: Message) -> Command<Message> {
        // ファイルがドロップされたときに、アプリの状態を変更する。
        // Eventのenumの中に、イベントの内容（別のEventのenum）とか、
        // 今回のFileDroppedではファイルパスが含まれたりする。
        match message {
            Message::EventOccurred(event) => {
                if let iced::event::Event::Window(we) = event {
                    if let iced::window::Event::FileDropped(path) = we {
                        self.path = path;

                        // read file, prepare data in buffer
                        self.files = Vec::new();
                        
                        // ファイル読み込み処理
                        match load_archive_file(&self.path, &mut self.buf) {
                            Ok(_) => log::info!("アーカイブファイルの読み込みが完了しました"),
                            Err(e) => {
                                log::error!("アーカイブファイルの読み込みに失敗: {}", e);
                                // エラー状態をリセット
                                self.files.clear();
                                self.buf.clear();
                                self.f_idx = 0;
                                self.f_max = 0;
                            }
                        }

                        // ファイルが正常に読み込まれた場合のみ処理を続行
                        if !self.buf.is_empty() {
                            match process_archive(&self.buf, &mut self.files, &mut self.f_idx, &mut self.f_max) {
                                Ok(_) => log::info!("アーカイブの処理が完了しました"),
                                Err(e) => {
                                    log::error!("アーカイブの処理に失敗: {}", e);
                                    self.f_idx = 0;
                                    self.f_max = 0;
                                }
                            }
                        }
                    }
                } else if let iced::event::Event::Keyboard(we) = event {
                    match we {
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Left,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            log::debug!("← キーが押されました");
                            if self.f_idx + 2 < self.f_max {
                                self.f_idx += 2;
                                log::debug!("ページを次に進めました: {}/{}", self.f_idx, self.f_max);
                            }
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Right,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            log::debug!("→ キーが押されました");
                            if self.f_idx > 2 {
                                self.f_idx -= 2;
                                log::debug!("ページを前に戻しました: {}/{}", self.f_idx, self.f_max);
                            }
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Up,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            log::debug!("↑ キーが押されました");
                            if self.f_idx > 1 {
                                self.f_idx -= 1;
                                log::debug!("ファイルインデックスを減らしました: {}/{}", self.f_idx, self.f_max);
                            }
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Down,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            log::debug!("↓ キーが押されました");
                            if self.f_idx + 1 < self.f_max {
                                self.f_idx += 1;
                                log::debug!("ファイルインデックスを増やしました: {}/{}", self.f_idx, self.f_max);
                            }
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Key1,
                            modifiers: _
                        } => {
                            self.page = Pages::Single;
                            log::info!("シングルページモードに切り替えました");
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Key2,
                            modifiers: _
                        } => {
                            self.page = Pages::Double;
                            log::info!("ダブルページモードに切り替えました");
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::R,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            log::debug!("R キーが押されました");
                            self.rotate = !self.rotate;
                            log::info!("回転モード: {}", if self.rotate { "ON" } else { "OFF" });
                        },
                        _ => {},
                    }
                }
            },
        };

        Command::none()
    }

    // イベントが発生したときに呼び出される。マウス操作、ウィンドウ関係、キーボード操作等。
    // 何らかのSubscriptionを返すことで、update()が実行される。
    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::events().map(Message::EventOccurred)
    }

    // 表示されるGUIを生成する。
    fn view(&self) -> Element<Message> {
        // ファイルパス表示部
        let mut p = self.path.to_str().unwrap_or("").to_string();
        if p.is_empty() {
            p = String::from("画像ファイルをウィンドウにドロップしてね。");
        }
        let path = Container::new(Text::new(p).size(20)).padding(4);

        // 画像表示部
        if //self.path.to_str().unwrap_or("").to_string().contains(".rar") && 
           self.files.len() > 0 {
            if matches!(self.page, Pages::Single) {
				view_single(&self)
            } else if matches!(self.page, Pages::Double) {
				view_double(&self)
            } else {
                let image_e = Container::new(Text::new("empty").size(20)).padding(4);

                let content = Column::new()
                    .width(Length::Fill)
                    .align_items(Alignment::Start)
                    .push(path)
                    .push(image_e);
    
                Container::new(content)
				.width(Length::Fill)
				.height(Length::Fill)
				.into()
            }
        } else {
            let image_e = Container::new(Text::new("empty").size(20)).padding(4);

			let content = Column::new()
				.width(Length::Fill)
				.align_items(Alignment::Start)
				.push(path)
				.push(image_e);

			Container::new(content)
				.width(Length::Fill)
				.height(Length::Fill)
				.into()
        }
    }

}

fn get_image_handle(ev: &Events, f_idx: usize) -> iced::widget::image::Handle {
    if ev.files.len() <= f_idx {
        log::warn!("無効なファイルインデックス: {} >= {}", f_idx, ev.files.len());
        return create_error_image();
    }

    let f = &ev.files[f_idx];
    log::debug!("描画中: {} (offset: {}, size: {}, fsize: {})", f.filepath, f.offset, f.size, f.fsize);

    let data = match decompress_file_data(ev, f) {
        Ok(data) => data,
        Err(e) => {
            log::error!("ファイルの解凍に失敗: {}", e);
            return create_error_image();
        }
    };

    match create_image_handle(&data, ev.rotate) {
        Ok(handle) => handle,
        Err(e) => {
            log::error!("画像の作成に失敗: {}", e);
            create_error_image()
        }
    }
}

fn view_single(ev: &Events) -> Element<Message> {
    let image_s;
    {
        let handle = get_image_handle(ev, ev.f_idx);

        image_s = Container::new(
            iced::widget::image::Viewer::new(handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);
    }

    let content = Column::new()
        .width(Length::Fill)
        .align_items(Alignment::Start)
        .push(image_s);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

// === エラーハンドリング用ヘルパー関数 ===

/// アーカイブファイルを読み込む
fn load_archive_file(path: &PathBuf, buf: &mut Vec<u8>) -> ArchiveResult<()> {
    use crate::archive_reader::{ArchiveResult, ArchiveError};
    
    let file_path = path.to_str()
        .ok_or_else(|| ArchiveError::CorruptedArchive {
            message: "無効なファイルパスです".to_string(),
        })?;

    let mut file = File::open(file_path)?;
    buf.clear();
    file.read_to_end(buf)?;
    
    log::info!("ファイルを読み込みました: {} ({} bytes)", file_path, buf.len());
    Ok(())
}

/// アーカイブを処理してファイルリストを作成
fn process_archive(buf: &[u8], files: &mut Vec<MemberFile>, f_idx: &mut usize, f_max: &mut usize) -> ArchiveResult<()> {
    use crate::archive_reader::{ArchiveResult, ArchiveError};
    
    let ftype = check_file_type(buf)?;
    
    match ftype {
        FileType::Rar5 => {
            log::info!("ファイル形式: RAR5");
            Rar5Reader::read_archive(buf, files)?
        },
        FileType::Rar4 => {
            log::info!("ファイル形式: RAR4");
            Rar4Reader::read_archive(buf, files)?
        },
        FileType::Zip => {
            log::info!("ファイル形式: ZIP");
            ZipReader::read_archive(buf, files)?
        },
        FileType::Unsupported => {
            return Err(ArchiveError::UnsupportedFormat);
        }
    }
    
    sort_filename(files);
    *f_idx = 0;
    *f_max = files.len();
    
    log::info!("アーカイブの処理が完了: {} 個のファイルを検出", files.len());
    Ok(())
}

/// ファイルデータを解凍
 fn decompress_file_data(ev: &Events, file: &MemberFile) -> ArchiveResult<Vec<u8>> {
    match file.ctype {
        CompressionType::Uncompress => {
            Rar5Reader::read_data(&ev.buf, file.offset, file.size)
        },
        CompressionType::Deflate | CompressionType::Deflate64 => {
            compress_deflate::uncomp_deflate(&ev.buf, file.offset, file.size)
        },
        CompressionType::Rar5 | CompressionType::Rar4 => {
            Err(ArchiveError::DecompressionError(
                "RAR圧縮はまだサポートされていません".to_string()
            ))
        },
        CompressionType::Unsupported => {
            Err(ArchiveError::DecompressionError(
                "サポートされていない圧縮形式です".to_string()
            ))
        }
    }
}

/// 画像ハンドルを作成
fn create_image_handle(data: &[u8], rotate: bool) -> ArchiveResult<iced::widget::image::Handle> {
    if rotate {
        let pimg = image::load_from_memory(data)?;
        let rotated = pimg.rotate180();
        let bytes = rotated.clone().into_rgba8().into_raw();
        Ok(iced::widget::image::Handle::from_pixels(
            rotated.width(),
            rotated.height(),
            bytes,
        ))
    } else {
        Ok(iced::widget::image::Handle::from_memory(data.to_vec()))
    }
}

/// エラー用の赤い画像を作成
fn create_error_image() -> iced::widget::image::Handle {
    let pimg = ImageBuffer::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
    iced::widget::image::Handle::from_pixels(
        pimg.width(),
        pimg.height(),
        pimg.into_vec(),
    )
}

fn view_double(ev: &Events) -> Element<Message> {
    let image_r;
    let image_l;

    let handle_1 = get_image_handle(ev, ev.f_idx);
    let handle_2 = get_image_handle(ev, ev.f_idx + 1);
    {
        let handle = if !ev.rotate {handle_1.clone()} else {handle_2.clone()};
        image_r = Container::new(
            iced::widget::image::Viewer::new(handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Left)
        .align_y(alignment::Vertical::Center);
    }

    {
        let handle = if !ev.rotate {handle_2.clone()} else {handle_1.clone()};
        image_l = Container::new(
            iced::widget::image::Viewer::new(handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Center);
    }

    let doubleview = Row::new()
        .width(Length::Fill)
        .align_items(Alignment::Start)
        .push(image_l)
        .push(image_r);

    let content = Column::new()
        .width(Length::Fill)
        .align_items(Alignment::Start)
        .push(doubleview);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
