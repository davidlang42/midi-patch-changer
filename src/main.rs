use std::fs;
use std::env;
use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::io::{Read, Write};
use console::{Term, Key};

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