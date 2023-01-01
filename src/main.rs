use std::env;
use std::error::Error;
use iced::{Application, Settings};

mod midi;
mod cli;
mod gui;

#[macro_use] extern crate serde_derive;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // cli
        let midi_in = args.get(1).ok_or("The first argument should be the MIDI-IN device (or '-' for no input device)")?;
        let midi_out = args.get(2).ok_or("The second argument should be the MIDI-OUT device")?;
        let mut device = midi::ThruDevice::new(if midi_in == "-" { None } else { Some(midi_in) }, midi_out, args.get(3))?;
        cli::run(&mut device);
    } else {
        // gui
        let flags = gui::devicepicker::Flags {//TODO is there a better way to reference this?
            options: vec![String::from("Test 1"), String::from("Test 2"), String::from("Test 3")],//TODO
            midi_in: None,
            midi_out: String::new(),
            patch_file: String::new()
        };
        gui::DevicePicker::run(Settings::with_flags(flags)).map_err(|e| format!("GUI error: {}", e))?;
        //println!()
        //gui::PatchSystem::run(Settings::with_flags(device)).map_err(|e| format!("GUI error: {}", e))?;
    }
    Ok(())
}