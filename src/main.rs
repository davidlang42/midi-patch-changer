use std::fs;
use std::env;
use std::error::Error;
use iced::{Application, Settings};
use std::sync::mpsc;
use std::process::Command;

mod midi;
mod cli;
mod gui;

#[macro_use] extern crate serde_derive;

enum Mode {
    Cli,
    Gui
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 3 {
        // cli/gui patches
        let mode = if args[1] == "cli" {
            Mode::Cli
        } else if args[1] == "gui" {
            Mode::Gui
        } else {
            return Err(format!("The first argument must specifiy 'cli' or 'gui'").into())
        };
        let midi_in = args.get(2).ok_or("The first argument should be the MIDI-IN device (or '-' for no input device)")?;
        let midi_out = args.get(3).ok_or("The second argument should be the MIDI-OUT device")?;
        let patch_file: Option<&str> = match args.get(4) {
            Some(file) => Some(file),
            None => None
        };
        let mut device = midi::ThruDevice::new(if midi_in == "-" { None } else { Some(midi_in) }, midi_out, patch_file)?;
        match mode {
            Mode::Cli => cli::run(&mut device),
            Mode::Gui => gui::PatchSystem::run(Settings::with_flags(device)).map_err(|e| format!("PatchSystem GUI error: {}", e))?
        }
    } else {
        // gui picker
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
        if let Ok(result) = rx.try_recv() {
            let mut new_args: Vec<String> = vec![
                String::from("gui"),
                match result.midi_in { Some(midi_in) => midi_in, None => String::from("-") },
                result.midi_out
            ];
            if let Some(patch_file) = result.patch_file {
                new_args.push(patch_file);
            }
            let _command = Command::new(&args[0]).args(new_args).spawn()?;
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
        files.sort();
        Ok(files)
    } else {
        Ok(vec![root.to_string()])
    }
}