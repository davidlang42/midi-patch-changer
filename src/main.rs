use std::fs;
use std::env;
use std::error::Error;
use iced::{Application, Settings};
use std::sync::mpsc;

mod midi;
mod cli;
mod gui;

#[macro_use] extern crate serde_derive;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 {
        // cli
        let midi_in = args.get(1).ok_or("The first argument should be the MIDI-IN device (or '-' for no input device)")?;
        let midi_out = args.get(2).ok_or("The second argument should be the MIDI-OUT device")?;
        let mut device = midi::ThruDevice::new(if midi_in == "-" { None } else { Some(midi_in) }, midi_out, args.get(3))?;
        cli::run(&mut device);
    } else {
        // gui
        let patch_dir_or_file = match args.get(1) {
            Some(arg) => arg,
            None => "."
        };
        let (tx, rx) = mpsc::channel();
        let flags = gui::devicepicker::Flags {
            midi_options: list_files("/dev", "midi")?,
            patch_options: list_files(patch_dir_or_file, "")?,
            result_sender: tx
        };
        gui::DevicePicker::run(Settings::with_flags(flags)).map_err(|e| format!("DevicePicker GUI error: {}", e))?;
        if let Ok(device) = rx.try_recv() {
            gui::PatchSystem::run(Settings::with_flags(device)).map_err(|e| format!("PatchSystem GUI error: {}", e))?;
        } else {
            println!("No devices selected.");
        }
    }
    Ok(())
}

fn list_files(root: &str, prefix: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let md = fs::metadata(root)?;
    if md.is_dir() {
        let mut files = Vec::new();
        for entry in fs::read_dir(root)? {
            let path = entry?.path();
            if !path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with(prefix) {
                files.push(path.display().to_string());
            }
        }
        Ok(files)
    } else {
        Ok(vec![root.to_string()])
    }
}