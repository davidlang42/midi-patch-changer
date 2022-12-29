use std::fs;
use std::env;
use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::io::{Read, Write};
// use wmidi::MidiMessage;

#[macro_use] extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct PatchPreset {
    name: String,
    patch: Option<u8>
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse arguments & patches
    let args: Vec<String> = env::args().collect();
    let midi_in = args.get(1).ok_or("The first argument should be the MIDI-IN device")?;
    let midi_out = args.get(2).ok_or("The second argument should be the MIDI-OUT device")?;
    let patches: Vec<PatchPreset> = match args.get(3) {
        Some(patch_file) => {
            let json = fs::read_to_string(&patch_file).map_err(|e| format!("Cannot read from '{}': {}", patch_file, e))?;
            //TODO hack in some nicety for trailing commas, newlines instead of commas, non-quoted keys
            serde_json::from_str(&format!("[{}]", json)).map_err(|e| format!("Cannot parse patches from '{}': {}", patch_file, e))?
        },
        None => Vec::new()
    };
    // open devices & initiate midi-thru
    let mut input = fs::File::options().read(true).open(midi_in).map_err(|e| format!("Cannot open MIDI IN '{}': {}", midi_in, e))?;
    let mut output = fs::File::options().write(true).open(midi_out).map_err(|e| format!("Cannot open MIDI OUT '{}': {}", midi_out, e))?;
    let (tx, rx) = mpsc::channel();
    thread::Builder::new().name(format!("midi-out")).spawn(move || write_from_queue(&mut output, rx))?;
    thread::Builder::new().name(format!("midi-in")).spawn(move || read_into_queue(&mut input, tx.clone()))?;
    // run patch change UI
    loop {
        //TODO
    }
}

fn read_into_queue(f: &mut fs::File, tx: mpsc::Sender<u8>) {
    let mut buf: [u8; 1] = [0; 1];
    while f.read_exact(&mut buf).is_ok() {
        if tx.send(buf[0]).is_err() {
            panic!("Error writing to queue.");
        }
    }
    panic!("Reading into queue has finished.");
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