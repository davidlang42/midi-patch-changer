use iced::widget::{button, row, column, text, text_input, pick_list};
use iced::{Application, Command, Theme, Element};
use iced::executor;
use iced::Subscription;
use iced_native::{window, Event};
use iced::window::set_mode;
use std::sync::mpsc;

pub struct DevicePicker {
    options: Vec<String>,
    midi_in: String,
    midi_out: String,
    patch_file: String,
    exit: bool,
    screen_width: u32,
    screen_height: u32,
    result_sender: mpsc::Sender<String>
}

pub struct Flags {
    pub options: Vec<String>,
    pub result_sender: mpsc::Sender<String>
}

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Quit,
    //PatchFileBrowse,
    PatchFileChanged(String),
    MidiInChanged(String),
    MidiOutChanged(String),
    EventOccurred(iced_native::Event)
}

impl Application for DevicePicker {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = Flags;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (Self {
            options: flags.options,
            result_sender: flags.result_sender,
            midi_in: String::new(),
            midi_out: String::new(),
            patch_file: String::new(),
            exit: false,
            screen_width: 100,
            screen_height: 100
        //TODO }, set_mode(window::Mode::Fullscreen))
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("MIDI Device Picker")
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::events().map(Message::EventOccurred)
    }

    fn view(&self) -> Element<Message> {
        let height = (self.screen_height / 3) as u16;
        let size = height / 3;
        column![
            row![
                text("MIDI IN: ").size(size),
                pick_list(&self.options, Some(self.midi_in.clone()), Message::MidiInChanged)
            ],
            row![
                text("MIDI OUT: ").size(size),
                pick_list(&self.options, Some(self.midi_out.clone()), Message::MidiOutChanged)
            ],
            row![
                text("Patches: ").size(size),
                text_input("Path to file", &self.patch_file, Message::PatchFileChanged)
            ],
            row![
                button("Start").on_press(Message::Start),
                button("Quit").on_press(Message::Quit)
            ]
        ].into()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::Resized { width, height }) = event {
                    self.screen_width = width;
                    self.screen_height = height;
                }
            },
            Message::Start => {
                self.result_sender.send(self.patch_file.clone());
                self.exit = true;
            },
            Message::Quit => self.exit = true,
            Message::MidiInChanged(midi_in) => {
                self.midi_in = midi_in;//TODO handle no selection
                //TODO avoid conflicts
            },
            Message::MidiOutChanged(midi_out) => {
                self.midi_out = midi_out;
                //TODO avoid conflicts
            },
            Message::PatchFileChanged(patch_file) => {
                self.patch_file = patch_file;
                //TODO what if its blank?
            }
        }
        Command::none()
    }
}