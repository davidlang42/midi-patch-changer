# Midi Patch Changer
A CLI and GUI tool which forwards MIDI messages from an input device to an output device (ie. MIDI-thru), while allowing patch change MIDI messages to be sent by incrementing through a predefined list

## Installation notes
On linux, this is required due to the [font-loader](https://crates.io/crates/font-loader) dependency:
```
sudo apt-get install libfontconfig libfontconfig1-dev
```

## Usage
### Device picker
To start the device picker GUI, run:
```
./midi_patch_changer [optional patch file or folder]
```
This will provide user input to choose the MIDI devices and patch file from a list. By default, every file in the current directory is listed as an option for the patch file, however if the first argument is a folder it will list every file in that directory instead. If a file is provided as the only argument, it will be the only available option in the patch file list.

Once the user has selected options, clicking "Start" will launch a new process with args as per "Patch system (GUI)" below.

### Patch system (GUI)
To start the patch system GUI, run:
```
./midi_patch_changer gui [midi in device] [midi out device] [patch file]
```
The first argument is to differentiate from "Patch system (CLI)" below. The second/third arguments can be any valid readable/writable (respectively) file/device. If no midi in device is wanted, you can use '-' instead. The last argument is optional and can be ommitted.

### Patch system (CLI)
For instances when a key press is more convenient than a mouse click, the original CLI tool (v1.0) can still be used by running:
```
./midi_patch_changer cli [midi in device] [midi out device] [patch file]
```
Much like "Patch system (GUI)" above, the first argument is to specify CLI mode, the second/third can be any valid files (or '-' for midi in), and the last is optional.