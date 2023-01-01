use std::sync::mpsc;
use std::fs;
use std::thread;
use std::io::{Read, Write};
use std::error::Error;

//TODO send proper midi messages to avoid race condition between patch change commands and midi-thru from input device
//use wmidi::MidiMessage;
//[dependencies]
//wmidi = "4.0.6"

#[derive(Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    bank_msb: Option<u8>,
    bank_lsb: Option<u8>,
    program: Option<u8>,
    midi: Option<String> // space separated hex bytes of arbitrary midi commands
}

impl Patch {
    pub fn send(&self, tx: &mpsc::Sender<u8>) {
        if let Some(msb) = self.bank_msb {
            tx.send(176).unwrap(); // B0
            tx.send(0).unwrap(); // 00
            tx.send(msb).unwrap();
        }
        if let Some(lsb) = self.bank_lsb {
            tx.send(176).unwrap(); // B0
            tx.send(32).unwrap(); // 20
            tx.send(lsb).unwrap();
        }
        if let Some(prog) = self.program {
            tx.send(192).unwrap(); // C0
            tx.send(prog).unwrap();
        }
        if let Some(midi_string) = &self.midi {
            for hex in midi_string.split(" ") {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    tx.send(byte).unwrap();
                } else {
                    println!("'{}' is not a valid HEX MIDI byte (00-FF)", hex);
                }
            }
        }
    }
}

pub struct ThruDevice {
    patch_sender: mpsc::Sender<u8>,
    patch_list: Vec<Patch>,
    patch_index: usize
}

impl ThruDevice {
    pub fn new(midi_in: Option<&String>, midi_out: &String, patch_file: Option<&String>) -> Result<Self, Box<dyn Error>> {
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