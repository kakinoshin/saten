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

mod readrar;

pub fn main() -> iced::Result {
    // フォントを指定しつつ実行する。
    Events::run(Settings {
        default_font: Some(include_bytes!("../fonts/NotoSansJP-Regular.ttf")),
        ..Settings::default()
    })
}

// メインとなる構造体。アプリで保持する状態を変数にする。
#[derive(Debug, Default)]
struct Events {
    path: PathBuf,
    files : Vec<readrar::MemberFile>,
    f_idx : usize,
    f_max : usize,
    buf   : Vec<u8>,
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
                            _ = readrar::read_rar(&self.buf, &mut self.files);
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
                            modifiers,
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
                            modifiers,
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
                            modifiers,
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
                            modifiers,
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
        let image_r;
        if self.path.to_str().unwrap_or("").to_string().contains(".rar") {
            let f = &self.files[self.f_idx];
            println!("Drawing : {}/{}/{}/{}", f.filepath, f.offset, f.size, f.fsize);
            let data = readrar::read_data(&self.buf, f.offset, f.size);
            let handle = iced::widget::image::Handle::from_memory(data);

            image_r = Container::new(
                iced::widget::image::Viewer::new(handle)
            )
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left)
            .align_y(alignment::Vertical::Center);
        } else {
            image_r = Container::new(
                Image::new(self.path.clone())
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left)
            .align_y(alignment::Vertical::Center);
        }

        let image_l;
        if self.path.to_str().unwrap_or("").to_string().contains(".rar") &&
           self.f_idx + 1 < self.f_max {
            let f = &self.files[self.f_idx+1];
            println!("Drawing(R) : {}/{}/{}/{}", f.filepath, f.offset, f.size, f.fsize);
            let data = readrar::read_data(&self.buf, f.offset, f.size);
            let handle = iced::widget::image::Handle::from_memory(data);

            image_l = Container::new(
                iced::widget::image::Viewer::new(handle)
            )
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .align_y(alignment::Vertical::Center);
        } else {
            image_l = Container::new(
                Image::new(self.path.clone())
                    .width(Length::Fill)
                    .height(Length::Fill),
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
            .push(path)
            .push(doubleview);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
}
