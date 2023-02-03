use std::sync::mpsc;
use std::fs;
use std::thread;
use std::io::{Read, Write};
use std::error::Error;
use wmidi::{MidiMessage, ControlFunction, FromBytesError, Channel, U7};

#[derive(Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    channel: Option<u8>,
    bank_msb: Option<u8>,
    bank_lsb: Option<u8>,
    program: Option<u8>
}

impl Patch {
    pub fn send(&self, tx: &mpsc::Sender<MidiMessage<'static>>) {
        let channel = Channel::from_index(match self.channel {
            Some(ch) if ch < 16 => ch,
            _ => 0
        }).unwrap();
        if let Some(msb) = self.bank_msb {
            tx.send(MidiMessage::ControlChange(channel, ControlFunction::BANK_SELECT, U7::from_u8_lossy(msb))).unwrap();
        }
        if let Some(lsb) = self.bank_lsb {
            tx.send(MidiMessage::ControlChange(channel, ControlFunction::BANK_SELECT_LSB, U7::from_u8_lossy(lsb))).unwrap();
        }
        if let Some(prog) = self.program {
            tx.send(MidiMessage::ProgramChange(channel, U7::from_u8_lossy(prog))).unwrap();
        }
    }
}

pub struct ThruDevice {
    patch_sender: mpsc::Sender<MidiMessage<'static>>,
    patch_list: Vec<Patch>,
    patch_index: usize
}

impl ThruDevice {
    pub fn new(midi_in: Option<&str>, midi_out: &str, patch_file: Option<&str>) -> Result<Self, Box<dyn Error>> {
        // load patches
        let patch_list: Vec<Patch> = match patch_file {
            Some(file) => {
                let json = fs::read_to_string(&file).map_err(|e| format!("Cannot read from '{}': {}", file, e))?;
                //TODO hack in some nicety for trailing commas, newlines instead of commas, non-quoted keys
                serde_json::from_str(&format!("[{}]", json)).map_err(|e| format!("Cannot parse patches from '{}': {}", file, e))?
            },
            None => Vec::new()
        };
        // open devices & initiate midi-thru
        let (tx, rx) = mpsc::channel();
        let mut output = fs::File::options().write(true).open(midi_out).map_err(|e| format!("Cannot open MIDI OUT '{}': {}", midi_out, e))?;
        thread::Builder::new().name(format!("midi-out")).spawn(move || write_from_queue(&mut output, rx))?;
        if let Some(input_file) = midi_in {
            let mut input = fs::File::options().read(true).open(input_file).map_err(|e| format!("Cannot open MIDI IN '{}': {}", input_file, e))?;
            let tx_clone = tx.clone();
            thread::Builder::new().name(format!("midi-in")).spawn(move || read_into_queue(&mut input, tx_clone))?;
        }
        // send first patch & return connected device
        let device = Self {
            patch_sender: tx,
            patch_list,
            patch_index: 0
        };
        device.resend_patch();
        Ok(device)
    }

    pub fn increment_patch(&mut self, delta: isize) -> Option<(usize, &Patch)> {
        let new_index = self.patch_index as isize + delta;
        if new_index < 0 {
            self.set_patch(0)
        } else {
            self.set_patch(new_index as usize)
        }
    }

    pub fn has_patches(&self) -> bool {
        self.patch_list.len() > 0
    }

    pub fn set_patch(&mut self, index: usize) -> Option<(usize, &Patch)> {
        if !self.has_patches() {
            return None;
        }
        let new_index = if index >= self.patch_list.len() {
            self.patch_list.len() - 1
        } else {
            index
        };
        if new_index != self.patch_index {
            self.patch_index = new_index;
            self.resend_patch();
            self.current_patch()
        } else {
            None
        }
    }

    fn resend_patch(&self) {
        if self.has_patches() {
            self.patch_list[self.patch_index].send(&self.patch_sender);
        }
    }

    pub fn current_patch(&self) -> Option<(usize, &Patch)> {
        if self.has_patches() {
            Some((self.patch_index + 1, &self.patch_list[self.patch_index]))
        } else {
            None
        }
    }

    pub fn next_patch(&self) -> Option<&Patch> {
        self.patch_list.get(self.patch_index + 1)
    }

    pub fn previous_patch(&self) -> Option<&Patch> {
        if self.patch_index == 0 {
            None
        } else {
            self.patch_list.get(self.patch_index - 1)
        }
    }
}

fn read_into_queue(f: &mut fs::File, tx: mpsc::Sender<MidiMessage>) {
    let mut buf: [u8; 1] = [0; 1];
    let mut bytes = Vec::new();
    while f.read_exact(&mut buf).is_ok() {
        bytes.push(buf[0]);
        match MidiMessage::try_from(bytes.as_slice()) {
            Ok(message) => {
                // message complete, send to queue
                if tx.send(message.to_owned()).is_err() {
                    panic!("Error sending to queue.");
                }
                bytes.clear();
            },
            Err(FromBytesError::NoBytes) | Err(FromBytesError::NoSysExEndByte) | Err(FromBytesError::NotEnoughBytes) => {
                // wait for more bytes
            }, 
            _ => {
                // invalid message, clear and wait for next message
                bytes.clear();
            }
        }
    }
    println!("NOTE: Input device is not connected.");
}

fn write_from_queue(f: &mut fs::File, rx: mpsc::Receiver<MidiMessage>) {
    let mut buf = Vec::new();
    for received in rx {
        let expected = received.bytes_size();
        buf.resize(expected, 0);
        match received.copy_to_slice(&mut buf) {
            Ok(found) if found != expected => panic!("Error writing midi message: Not enough bytes (expected {} found {}).", expected, found),
            Err(_) => panic!("Error writing midi message: Too many bytes (expected {}).", expected),
            _ => {}
        }
        if f.write_all(&buf).is_err() {
            panic!("Error writing to device.")
        }
        if f.flush().is_err() {
            panic!("Error flushing to device.");
        }
    }
    panic!("Writing from queue has finished.");
}
