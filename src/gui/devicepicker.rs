use iced::widget::{button, row, column, text, pick_list};
use iced::{Application, Command, Theme, Element};
use iced::executor;
use iced::Subscription;
use iced_native::{window, Event};
//use iced::window::set_mode;
use std::sync::mpsc;

pub struct DevicePicker {
    midi_in_options: Vec<String>,
    midi_in: String,
    midi_out_options: Vec<String>,
    midi_out: String,
    patch_options: Vec<String>,
    patch_file: String,
    exit: bool,
    screen_width: u32,
    screen_height: u32,
    result_sender: mpsc::Sender<DeviceResult>,
    last_error: String
}

pub struct Flags {
    pub midi_options: Vec<String>,
    pub patch_options: Vec<String>,
    pub result_sender: mpsc::Sender<DeviceResult>
}

pub struct DeviceResult {
    pub midi_in: Option<String>,
    pub midi_out: String,
    pub patch_file: Option<String>,
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
        let (midi_in, midi_out) = match flags.midi_options.len() {
            0 => panic!("No MIDI devices found."),
            1 => (String::new(), flags.midi_options[0].clone()),
            _ => (flags.midi_options[0].clone(), flags.midi_options[1].clone())
        };
        let patch_file = match flags.patch_options.len() {
            0 => String::new(),
            _ => flags.patch_options[0].clone()
        };
        let mut midi_in_options = flags.midi_options.clone();
        midi_in_options.insert(0, String::new());
        let mut patch_options = flags.patch_options;
        patch_options.insert(0, String::new());
        (Self {
            midi_in_options,
            midi_out_options: flags.midi_options,
            patch_options,
            result_sender: flags.result_sender,
            midi_in,
            midi_out,
            patch_file,
            exit: false,
            screen_width: 100,
            screen_height: 100,
            last_error: String::new()
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
                pick_list(&self.midi_in_options, Some(self.midi_in.clone()), Message::MidiInChanged)
            ],
            row![
                text("MIDI OUT: ").size(size),
                pick_list(&self.midi_out_options, Some(self.midi_out.clone()), Message::MidiOutChanged)
            ],
            row![
                text("Patches: ").size(size),
                pick_list(&self.patch_options, Some(self.patch_file.clone()), Message::PatchFileChanged)
            ],
            row![
                button("Start").on_press(Message::Start),
                button("Quit").on_press(Message::Quit)
            ],
            text(&self.last_error).size(size / 2)
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
                if !self.exit {
                    let result = DeviceResult {
                        midi_in: non_empty(self.midi_in.clone()),
                        midi_out: self.midi_out.clone(),
                        patch_file: non_empty(self.patch_file.clone())
                    };
                    if let Err(e) = self.result_sender.send(result) {
                        self.last_error = format!("{}", e);
                    } else {
                        self.exit = true;
                    }
                }
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

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}