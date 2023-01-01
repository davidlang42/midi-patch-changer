# Midi Patch Changer
A CLI tool which forwards MIDI messages from an input device to an output device (ie. MIDI-thru), while allowing patch change MIDI messages to be sent by incrementing through a predefined list

## Installation notes
On linux, this is required due to the [font-loader](https://crates.io/crates/font-loader) dependency:
```
sudo apt-get install libfontconfig libfontconfig1-dev
```
