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
mod archive_reader;

use crate::reader_rar5::Rar5Reader;
use crate::archive_reader::ArcReader;
use crate::archive_reader::MemberFile;

pub fn main() -> iced::Result {
    // フォントを指定しつつ実行する。
    Events::run(Settings {
        default_font: Some(include_bytes!("../fonts/NotoSansJP-Regular.ttf")),
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
    path: PathBuf,
    files : Vec<MemberFile>,
    f_idx : usize,
    f_max : usize,
    buf   : Vec<u8>,
    page  : Pages,
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

                        // read rar file
                        if self.path.to_str().unwrap_or("").to_string().contains(".rar") {
                            self.files = Vec::new();
                            //readrar::read_rar(self.path.to_str().unwrap(), &mut self.files);
                            let mut file = match File::open(self.path.to_str().unwrap()) {
                                Ok(f) => f,
                                Err(err) => panic!("file error: {}", err)
                            };
                            self.buf = Vec::new();
                            let _ = file.read_to_end(&mut self.buf);
                            _ = Rar5Reader::read_archive(&self.buf, &mut self.files);
                            self.files.sort_by(|a, b| a.filepath.to_lowercase().cmp(&b.filepath.to_lowercase()));
                            for f in &self.files {
                                println!("{}/{}/{}/{}", f.filepath, f.offset, f.size, f.fsize);
                            }
                            self.f_idx = 0;
                            self.f_max = self.files.len();
                        } else {
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
        println!("view {}", matches!(self.page, Pages::Double));
        // ファイルパス表示部
        let mut p = self.path.to_str().unwrap_or("").to_string();
        if p.is_empty() {
            p = String::from("画像ファイルをウィンドウにドロップしてね。");
        }
        let path = Container::new(Text::new(p).size(20)).padding(4);

        // 画像表示部
        if self.path.to_str().unwrap_or("").to_string().contains(".rar") && 
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

fn view_single(ev: &Events) -> Element<Message> {
    let image_s;
    {
        let f = &ev.files[ev.f_idx];
        println!("Drawing : {}/{}/{}/{}", f.filepath, f.offset, f.size, f.fsize);

        let mut start = Instant::now();
        let data = Rar5Reader::read_data(&ev.buf, f.offset, f.size);
        let mut end = start.elapsed();
        println!("read file takes {}.{:03}sec ", end.as_secs(), end.subsec_nanos() / 1_000_000);

        start = Instant::now();
        let handle = iced::widget::image::Handle::from_memory(data);

        image_s = Container::new(
            iced::widget::image::Viewer::new(handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);
        end = start.elapsed();
        println!("draw left image takes {}.{:03}sec ", end.as_secs(), end.subsec_nanos() / 1_000_000);
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
    println!("Drawing(DoubleView)");

    let image_r;
    let image_l;
    {
        let f = &ev.files[ev.f_idx];
        println!("Drawing : {}/{}/{}/{}", f.filepath, f.offset, f.size, f.fsize);
    
        let mut start = Instant::now();
        let data = Rar5Reader::read_data(&ev.buf, f.offset, f.size);
        let mut end = start.elapsed();
        println!("read file takes {}.{:03}sec ", end.as_secs(), end.subsec_nanos() / 1_000_000);

        start = Instant::now();
        let handle = iced::widget::image::Handle::from_memory(data);

        image_r = Container::new(
            iced::widget::image::Viewer::new(handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Left)
        .align_y(alignment::Vertical::Center);
        end = start.elapsed();
        println!("draw left image takes {}.{:03}sec ", end.as_secs(), end.subsec_nanos() / 1_000_000);
    }

    {
        let f = &ev.files[ev.f_idx+1];
        println!("Drawing(R) : {}/{}/{}/{}", f.filepath, f.offset, f.size, f.fsize);

        let mut start = Instant::now();
        let data = Rar5Reader::read_data(&ev.buf, f.offset, f.size);
        let mut end = start.elapsed();
        println!("read file takes {}.{:03}sec ", end.as_secs(), end.subsec_nanos() / 1_000_000);

        start = Instant::now();
        let handle = iced::widget::image::Handle::from_memory(data);

        image_l = Container::new(
            iced::widget::image::Viewer::new(handle)
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Center);
        end = start.elapsed();
        println!("draw left image takes {}.{:03}sec ", end.as_secs(), end.subsec_nanos() / 1_000_000);
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
