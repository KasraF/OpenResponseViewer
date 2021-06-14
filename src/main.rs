use iced::{
    button,
    container::{Style, StyleSheet},
    executor, Align, Application, Button, Checkbox, Clipboard, Column, Command, Container, Element,
    Settings, Subscription, Text, TextInput,
};
use iced_native::{keyboard, Event};
use serde::{Deserialize, Serialize};
use std::{cmp::min, fs::File, path::Path};

type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone)]
pub enum Message {
    NextRow,
    PrevRow,
    Input(String),
    Matches(bool),
    ToggleMatches,
    Ignore,
}

struct AppStyle {}
impl StyleSheet for AppStyle {
    fn style(&self) -> Style {
        Style {
            text_color: Some(iced::Color::from_rgb8(0x83, 0x94, 0x96)),
            background: Some(iced::Background::Color(iced::Color::from_rgb8(
                0x00, 0x2B, 0x36,
            ))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: iced::Color::TRANSPARENT,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    index: u32,
    lab: String,
    group: String,
    response: String,
    ratings: Vec<String>,
    matches: Option<bool>,
    codes: Option<String>,
}

struct Viewer {
    // metadata
    input_file_path: Box<Path>,
    output_file_path: Box<Path>,

    // The actual state
    idx: usize,

    // The rows
    data: Vec<Entry>,

    // The local state of the two buttons
    next_btn: button::State,
    prev_btn: button::State,
    codes_input_state: iced::text_input::State,
}

impl Viewer {
    fn save(&self) -> Result<(), Error> {
        let file = File::create(&self.output_file_path)?;
        serde_json::to_writer(file, &self.data)?;
        Ok(())
    }

    fn curr(&self) -> &Entry {
        &self.data[self.idx]
    }

    fn curr_mut(&mut self) -> &mut Entry {
        &mut self.data[self.idx]
    }
}

impl Application for Viewer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let args: Vec<String> = std::env::args().collect();

        assert_eq!(args.len(), 3);

        let file_path = Path::new(&args[1]);
        let output_file_path = Path::new(&args[2]);
        let file = std::fs::File::open(&file_path).expect(&format!(
            "Could not open file: {}",
            file_path.to_str().get_or_insert(&args[1])
        ));
        let data: Vec<Entry> = serde_json::from_reader(file).expect("Parsing json...");

        (
            Self {
                input_file_path: file_path.into(),
                output_file_path: output_file_path.into(),
                idx: 0,
                data,
                next_btn: button::State::default(),
                prev_btn: button::State::default(),
                codes_input_state: iced::text_input::State::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        let curr = &self.curr();
        format!(
            "Response Viewer - {}, {}, #{}",
            curr.lab, curr.group, curr.index
        )
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::NextRow => self.idx = min(self.idx + 1, self.data.len() - 1),
            Message::PrevRow => self.idx = self.idx.saturating_sub(1),
            // TODO REALLY need to do better error handling...
            Message::Input(input) => {
                self.curr_mut().codes = Some(input);
                self.save().expect("Saving file");
            }
            Message::Matches(matches) => {
                self.curr_mut().matches = Some(matches);
                self.save().expect("Saving file");
            }
            Message::ToggleMatches => {
                self.curr_mut().matches = self.curr_mut().matches.or(Some(false)).map(|b| !b);
                self.save().expect("Saving file");
            }
            Message::Ignore => (),
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(|event| match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Right,
                ..
            }) => Message::NextRow,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Left,
                ..
            }) => Message::PrevRow,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Space,
                ..
            }) => Message::ToggleMatches,
            _ => Message::Ignore,
        })
    }

    fn view(&mut self) -> Element<Message> {
        let buttons = iced::Row::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::End)
            .width(iced::Length::Fill)
            .push(Button::new(&mut self.prev_btn, Text::new("Prev")).on_press(Message::PrevRow))
            .push(Button::new(&mut self.next_btn, Text::new("Next")).on_press(Message::NextRow));

        let footer = iced::Row::new()
            .height(iced::Length::Fill)
            .width(iced::Length::Fill)
            .align_items(Align::End)
            .push(buttons)
            .push(iced::Text::new(format!(
                "{} / {}",
                self.idx + 1,
                self.data.len()
            )));

        let title = iced::Row::new().padding(10).spacing(10).push(Text::new({
            let row = &self.data[self.idx];
            format!("{}, {}, {}", row.lab, row.group, row.index)
        }));

        let mut ratings = iced::Column::new().padding(10);
        for rating in &self.data[self.idx].ratings {
            ratings = ratings.push(Text::new(rating));
        }

        let text = iced::Row::new()
            .padding(10)
            .push(Text::new(&self.data[self.idx].response));

        let input = iced::Row::new()
            .padding(10)
            .push(Checkbox::new(
                *self.data[self.idx].matches.get_or_insert(false),
                "Matches",
                Message::Matches,
            ))
            .spacing(10)
            .push(TextInput::new(
                &mut self.codes_input_state,
                "Codes",
                &self.data[self.idx].codes.get_or_insert("".to_string()),
                Message::Input,
            ));

        let content = Column::new()
            .padding(20)
            .push(title)
            .push(ratings)
            .push(input)
            .push(text)
            .push(footer);

        let container = Container::new(content).style(AppStyle {});

        container.into()
    }
}

fn main() -> iced::Result {
    Viewer::run(Settings {
        antialiasing: true,
        default_text_size: 18,
        ..Settings::default()
    })
}
