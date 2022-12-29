use std::fs;
use std::env;
use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::io::{Read, Write};
use console::{Term, Key};
//TODO send proper midi messages
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
    let tx_clone = tx.clone();
    thread::Builder::new().name(format!("midi-out")).spawn(move || write_from_queue(&mut output, rx))?;
    thread::Builder::new().name(format!("midi-in")).spawn(move || read_into_queue(&mut input, tx_clone))?;
    // run patch change UI
    let term = Term::stdout();
    let mut p = 0;
    if patches.len() == 0 {
        println!("**NO PATCHES**");
    } else {
        println!("{}", send_patch(&patches, p, &tx));
    }
    while let Ok(k) = term.read_key() {
        match k {
            _ if patches.len() == 0 => {
                println!("**NO PATCHES**");
            },
            Key::Backspace if p == 0 => {
                println!("**FIRST PATCH**");
            },
            Key::Backspace => {
                p -= 1;
                println!("<<< {}", send_patch(&patches, p, &tx));
            },
            _ if p == patches.len() - 1 => {
                println!("**LAST PATCH**");
            },
            _ => {
                p += 1;
                println!("{}", send_patch(&patches, p, &tx));
            }
        };
    }
    Ok(())
}

fn send_patch(patches: &Vec<PatchPreset>, index: usize, tx: &mpsc::Sender<u8>) -> String {
    let patch = &patches[index];
    if let Some(number) = patch.patch {
        if tx.send(192).is_err() {
            return format!("Error sending patch change #{}", index + 1);
        }
        if tx.send(number).is_err() {
            return format!("Error sending patch change #{}", index + 1);
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