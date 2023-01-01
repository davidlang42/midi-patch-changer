use crate::midi;
use console::{Term, Key};

pub fn run(device: &mut midi::ThruDevice) {
    let term = Term::stdout();
    match device.current_patch() {
        Some((number, patch)) => println!("#{} {}", number, patch.name),
        None => println!("**NO PATCHES**")
    };
    while let Ok(k) = term.read_key() {
        match k {
            _ if !device.has_patches() => println!("**NO PATCHES**"),
            Key::Backspace => match device.increment_patch(-1) {
                Some((number, patch)) => println!("<<< #{} {}", number, patch.name),
                None => println!("**FIRST PATCH**")
            },
            _ => match device.increment_patch(1) {
                Some((number, patch)) => println!("#{} {}", number, patch.name),
                None => println!("**LAST PATCH**")
            }
        };
    }
}