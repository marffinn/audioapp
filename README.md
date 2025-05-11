# Audio Controller

A simple, lightweight desktop application for Windows that allows you to easily control your audio devices and volume.

![Audio Controller Screenshot](screenshot.png)

## Features

- List and switch between all available audio output devices
- Control system volume with a slider
- Mute/unmute audio with a single click
- Minimalist, floating interface that stays on top of other windows
- Draggable window for easy positioning
- No command window visible during operation

## Requirements

- Windows 10 or later
- Rust (for building from source)

## Installation

### Option 1: Download the pre-built executable

1. Download the latest release from the [Releases](https://github.com/yourusername/audioapp/releases) page
2. Extract the ZIP file
3. Run `audioapp2.exe`

### Option 2: Build from source

1. Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/)
2. Clone this repository:
   ```
   git clone https://github.com/yourusername/audioapp.git
   cd audioapp
   ```
3. Build the application in release mode:
   ```
   cargo build --release
   ```
4. The executable will be available at `target/release/audioapp2.exe`

## Usage

1. Launch the application
2. The app will appear as a small floating window on your desktop
3. Select an audio device from the dropdown to set it as your default output
4. Use the slider to adjust the volume
5. Click the 🎵/🔇 button to toggle mute
6. Drag the title bar to move the window around
7. Click the X button to close the application

## Advanced Usage

### Setting as a Startup Application

To have Audio Controller start automatically when you log in to Windows:

1. Press `Win + R` to open the Run dialog
2. Type `shell:startup` and press Enter
3. Create a shortcut to `audioapp2.exe` in this folder

### Command Line Options

Currently, there are no command line options available.

## Troubleshooting

### Audio Device Switching Not Working

For the audio device switching functionality to work optimally, you may need to install the PowerShell AudioDeviceCmdlets module:

1. Open PowerShell as Administrator
2. Run: `Install-Module -Name AudioDeviceCmdlets`
3. When prompted, type 'Y' to install

If you don't have this module installed, the app will try alternative methods to switch devices, but they may not work in all cases.

## Building for Development

If you want to modify the application:

1. Clone the repository
2. Make your changes to the source code
3. Test with `cargo run`
4. Build with `cargo build --release`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [egui](https://github.com/emilk/egui) for the UI
- Uses [cpal](https://github.com/RustAudio/cpal) for audio device enumeration
- Uses [windows-volume-control](https://github.com/Waayway/windows-volume-control) for volume control
