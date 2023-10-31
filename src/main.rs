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
use crate::archive_reader::ArcReader;
use crate::archive_reader::MemberFile;
use crate::archive_reader::CompressionType;
use crate::file_checker::FileType;
use crate::file_checker::check_file_type;
use crate::sort_filename::sort_filename;

//use photon_rs::native::{open_image_from_bytes};
//use photon_rs::transform::rotate;

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
                        //readrar::read_rar(self.path.to_str().unwrap(), &mut self.files);
                        let mut file = match File::open(self.path.to_str().unwrap()) {
                            Ok(f) => f,
                            Err(err) => panic!("file error: {}", err)
                        };
                        self.buf = Vec::new();
                        let _ = file.read_to_end(&mut self.buf);

                        // check file format
                        let ftype = check_file_type(&self.buf);

                        // read file
                        if matches!(ftype, FileType::Rar5) {
                            println!("DEBUG: File type is RAR5");
                            _ = Rar5Reader::read_archive(&self.buf, &mut self.files);
                            sort_filename(&mut self.files);
                            self.f_idx = 0;
                            self.f_max = self.files.len();
                        } else if matches!(ftype, FileType::Rar4) {
                            println!("DEBUG: File type is RAR4");
                            _ = Rar4Reader::read_archive(&self.buf, &mut self.files);
                            sort_filename(&mut self.files);
                            self.f_idx = 0;
                            self.f_max = self.files.len();
                        } else if matches!(ftype, FileType::Zip) {
                            println!("DEBUG: File type is ZIP");
                            _ = ZipReader::read_archive(&self.buf, &mut self.files);
                            sort_filename(&mut self.files);
                            self.f_idx = 0;
                            self.f_max = self.files.len();
                        } else {
                            println!("DEBUG: Unsupported file");
                            self.f_idx = 0;
                            self.f_max = 0;
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
                            //     widget::focus_next()
                            // }
                            println!("Left");
                            if self.f_idx + 2 < self.f_max {
                                self.f_idx += 2;
                            }
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Right,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            //     widget::focus_next()
                            // }
                            println!("Right");
                            if self.f_idx > 2 {
                                self.f_idx -= 2;
                            }
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Up,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            //     widget::focus_next()
                            // }
                            println!("Up");
                            if self.f_idx > 1 {
                                self.f_idx -= 1;
                            }
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Down,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            //     widget::focus_next()
                            // }
                            println!("Down");
                            if self.f_idx + 1 < self.f_max {
                                self.f_idx += 1;
                            }
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Key1,
                            modifiers: _
                        } => {
                            self.page = Pages::Single;
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::Key2,
                            modifiers: _
                        } => {
                            self.page = Pages::Double;
                        },
                        iced::keyboard::Event::KeyPressed {
                            key_code: iced::keyboard::KeyCode::R,
                            modifiers: _
                        } => {
                            // if modifiers.shift() {
                            //     widget::focus_previous()
                            // } else {
                            //     widget::focus_next()
                            // }
                            println!("R");
                            self.rotate = !self.rotate;
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

fn get_image_handle(ev: &Events, f_idx : usize) -> iced::widget::image::Handle {
    let handle : iced::widget::image::Handle;
    if ev.files.len() > f_idx {
        let f = &ev.files[f_idx];
        println!("Drawing : {}/{}/{}/{}", f.filepath, f.offset, f.size, f.fsize);

        let data = match f.ctype {
            CompressionType::Uncompress => Rar5Reader::read_data(&ev.buf, f.offset, f.size),
            CompressionType::Deflate => compress_deflate::uncomp_deflate(&ev.buf, f.offset, f.size),
            _ => Vec::new(),
        };

        if ev.rotate {
            let pimg = image::load_from_memory(&data[..]).unwrap();
            let pimg = pimg.rotate180();
            let bytes = pimg.clone().into_rgba8().into_raw();
            handle = iced::widget::image::Handle::from_pixels(
                pimg.width() as u32,
                pimg.height() as u32,
            bytes,
            );
        } else {
            handle = iced::widget::image::Handle::from_memory(data);
        }
    } else {
        let pimg = ImageBuffer::from_pixel(16, 16, image::Rgba([255, 0, 0, 255]));
        handle = iced::widget::image::Handle::from_pixels(
            pimg.width() as u32,
            pimg.height() as u32,
            pimg.into_vec(),
        );

        // let width = 400;
        // let height = 300;
        // let mut imgbuf = ImageBuffer::new(width, height);

        // // Fill the ImageBuffer with some data (for example, a solid color)
        // for pixel in imgbuf.pixels_mut() {
        //     *pixel = image::Rgba([0, 255, 0, 255]); // Green
        // }

        // // Convert the ImageBuffer to bytes
        // let bytes: Vec<u8> = imgbuf.into_raw();

        // // Create an Iced Image handle
        // handle = iced::widget::image::Handle::from_pixels(width as u32, height as u32, bytes);
    }

    handle
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
