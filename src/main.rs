use std::fs;
use std::env;
use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::io::{Read, Write};

//TODO send proper midi messages to avoid race condition between patch change commands and midi-thru from input device
//use wmidi::MidiMessage;
//[dependencies]
//wmidi = "4.0.6"

#[macro_use] extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct Patch {
    name: String,
    bank_msb: Option<u8>,
    bank_lsb: Option<u8>,
    program: Option<u8>,
    midi: Option<String> // space separated hex bytes of arbitrary midi commands
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse arguments & patches
    let args: Vec<String> = env::args().collect();
    let midi_in = args.get(1).ok_or("The first argument should be the MIDI-IN device")?;
    let midi_out = args.get(2).ok_or("The second argument should be the MIDI-OUT device")?;
    let patches: Vec<Patch> = match args.get(3) {
        Some(patch_file) => {
            let json = fs::read_to_string(&patch_file).map_err(|e| format!("Cannot read from '{}': {}", patch_file, e))?;
            //TODO hack in some nicety for trailing commas, newlines instead of commas, non-quoted keys
            serde_json::from_str(&format!("[{}]", json)).map_err(|e| format!("Cannot parse patches from '{}': {}", patch_file, e))?
        },
        None => Vec::new()
    };
    // open devices & initiate midi-thru
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();
    let mut output = fs::File::options().write(true).open(midi_out).map_err(|e| format!("Cannot open MIDI OUT '{}': {}", midi_out, e))?;
    thread::Builder::new().name(format!("midi-out")).spawn(move || write_from_queue(&mut output, rx))?;
    if midi_in != "-" {
        let mut input = fs::File::options().read(true).open(midi_in).map_err(|e| format!("Cannot open MIDI IN '{}': {}", midi_in, e))?;
        thread::Builder::new().name(format!("midi-in")).spawn(move || read_into_queue(&mut input, tx_clone))?;
    }
    // run patch change UI
    PatchState::run(Settings::with_flags(InitialFlags {
        patches,
        tx
    })).map_err(|e| format!("GUI error: {}", e))?;
    Ok(())
}

fn send_patch(patches: &Vec<Patch>, index: usize, tx: &mpsc::Sender<u8>) -> String {
    let patch = &patches[index];
    if let Some(msb) = patch.bank_msb {
        tx.send(176).unwrap(); // B0
        tx.send(0).unwrap(); // 00
        tx.send(msb).unwrap();
    }
    if let Some(lsb) = patch.bank_lsb {
        tx.send(176).unwrap(); // B0
        tx.send(32).unwrap(); // 20
        tx.send(lsb).unwrap();
    }
    if let Some(prog) = patch.program {
        tx.send(192).unwrap(); // C0
        tx.send(prog).unwrap();
    }
    if let Some(midi_string) = &patch.midi {
        for hex in midi_string.split(" ") {
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                tx.send(byte).unwrap();
            } else {
                println!("Error: '{}' is not a valid HEX MIDI byte (00-FF)", hex);
            }
        }
    }
    format!("#{} {}", index + 1, patch.name)
}

fn read_into_queue(f: &mut fs::File, tx: mpsc::Sender<u8>) {
    let mut buf: [u8; 1] = [0; 1];
    while f.read_exact(&mut buf).is_ok() {
        if tx.send(buf[0]).is_err() {
            panic!("Error writing to queue.");
        }
    }
    println!("NOTE: Input device is not connected.");
}

fn write_from_queue(f: &mut fs::File, rx: mpsc::Receiver<u8>) {
    for received in rx {
        if f.write_all(&[received]).is_err() {
            panic!("Error writing to device.")
        }
        if f.flush().is_err() {
            panic!("Error flushing to device.");
        }
    }
    panic!("Writing from queue has finished.");
}

// GUI

struct InitialFlags {
    patches: Vec<Patch>,
    tx: mpsc::Sender<u8>
}

struct PatchState {
    patches: Vec<Patch>,
    index: usize,
    tx: mpsc::Sender<u8>,
    screen_height: u32,
    screen_width: u32,
    show_buttons: bool
}

impl PatchState {
    fn current(&self) -> String {
        if self.patches.len() > 0 {
            format!("#{} {}", self.index + 1, self.patches[self.index].name)
        } else {
            String::from("")
        }
    }

    fn next(&self) -> String {
        if self.index < self.patches.len() - 1 {
            format!("{}", self.patches[self.index + 1].name)
        } else {
            String::from("")
        }
    }

    fn previous(&self) -> String {
        if self.index > 0 {
            format!("{}", self.patches[self.index - 1].name)
        } else {
            String::from("")
        }
    }

    fn change(&mut self, delta: isize) {
        let new_index = self.index as isize + delta;
        if new_index >= 0 && (new_index as usize) < self.patches.len() {
            self.index = new_index as usize;
            self.resend();
        }
    }

    fn reset(&mut self) {
        self.index = 0;
        if self.patches.len() > 0 {
            self.resend();
        }
    }

    fn resend(&self) {
        send_patch(&self.patches, self.index, &self.tx);
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    NextPatch,
    PreviousPatch,
    ResetPatch,
    EventOccurred(iced_native::Event)
}

use iced::widget::{button, row, column, text};
use iced::{Application, Command, Theme, Settings, Element, Alignment};
use iced::executor;
use iced::Subscription;
use iced_native::{window, mouse, Event, Length};

impl Application for PatchState {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = InitialFlags;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let command = Command::none();
        //TODO let command = window::set_mode(window::Mode::Fullscreen);
        (Self { patches: flags.patches, index: 0, tx: flags.tx, screen_height: 100, screen_width: 100, show_buttons: false }, command)
    }

    fn title(&self) -> String {
        String::from("MIDI Patch Changer") //TODO include input/output device names & patch list name?
    }

    fn view(&self) -> Element<Message> {
        let mut col = column![
            text(self.previous()).size(20),
            text(self.current()).size(50),
            text(self.next()).size(20)
        ]
        .align_items(Alignment::Fill)
        .height(Length::Units(self.screen_height as u16))
        .width(Length::Units(self.screen_width as u16));
        if self.show_buttons {
            col = col.push(row![
                button("Next Patch").on_press(Message::NextPatch),
                button("Previous Patch").on_press(Message::PreviousPatch),
                button("Reset").on_press(Message::ResetPatch)
            ]);
        }
        col.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::subscription::events().map(Message::EventOccurred)
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::NextPatch => self.change(1),
            Message::PreviousPatch => self.change(-1),
            Message::ResetPatch => self.reset(),
            Message::EventOccurred(event) => {
                match event {
                    Event::Window(window::Event::Resized { width, height }) => {
                        self.screen_width = width;
                        self.screen_height = height;
                    },
                    Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                        self.show_buttons = false;
                        match button {
                            mouse::Button::Left => self.change(1),
                            mouse::Button::Right => self.change(-1),
                            _ => self.show_buttons = true
                        }
                    },
                    _ => {}
                }
            }
        }
        Command::none()
    }
}