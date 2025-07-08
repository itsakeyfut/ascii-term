# ASCII Terminal Media Player

[![Status](https://img.shields.io/badge/status-active--development-brightgreen?style=flat-square)]()
[![Rust Version](https://img.shields.io/badge/rust-1.87+-blue.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![bad_apple](./gallery/bad_apple.gif)

![cookie_bomb_rush](./gallery/cookie_bomb_rush.gif)

![yuruyuri](./gallery/yuruyuri.gif)

See More ⇒ [Gallery](./gallery/gallery.md)

- [Japanese](./README-JA.md)

A Rust-based terminal media player that converts and plays media files (video, audio, images) as ASCII art in real-time, featuring synchronized audio playback and colorful terminal output.

## Overview

This project demonstrates advanced media processing capabilities in Rust, showcasing real-time video decoding, ASCII art generation, and synchronized audio playback in terminal environments. Built as a modular architecture with clear separation of concerns, it serves as both a functional media player and a foundation for future video editing software development.

## Key Features

### Media Playback

- **Multi-format support**: MP4, AVI, MKV, MOV, MP3, FLAC, WAV, JPG, PNG, and more
- **Real-time ASCII rendering**: High-quality video-to-ASCII conversion with 10 character map options
- **Synchronized audio playback**: Frame-accurate audio-video synchronization using Rodio
- **Color ASCII art**: RGB color information preserved in colorful terminal output
- **Dynamic terminal adaptation**: Live resizing support with automatic scaling
- **Interactive controls**: Play/pause/stop/mute with keyboard shortcuts
- **Loop playback**: Continuous media replay functionality

### Character Map Variety

| Index | Name           | Description                     |
| ----- | -------------- | ------------------------------- |
| 0     | Basic ASCII    | 10 characters (` .:-=+*#%@`)    |
| 1     | Extended ASCII | Extended 67-character set       |
| 2     | Full ASCII     | Complete 92-character set       |
| 3     | Unicode Blocks | Block characters (` ░▒▓█`)      |
| 4     | Braille        | Braille dot patterns            |
| 5     | Dots           | Dot-based characters            |
| 6     | Gradient       | Gradient block characters       |
| 7     | Binary         | Binary (black/white) characters |
| 8     | Binary Dots    | Binary dot patterns             |
| 9     | Emoji Style    | Emoji-style characters          |

### Technical Architecture

- **Modular design**: Clear separation between core media processing (`media-core`) and UI presentation (`terminal-player`)
- **Safety-first approach**: Extensive use of FFmpeg and OpenCV wrapper crates for robust development
- **Multi-threaded processing**: Independent threads for video decoding, audio processing, and terminal rendering
- **Memory-efficient streaming**: Controlled buffering with optimized memory usage
- **Cross-platform compatibility**: Works on Linux, macOS, and Windows terminals

## Project Structure

```
├── Cargo.toml              # Workspace configuration
├── media-core/             # Core media processing library
│   ├── src/
│   │   ├── video/          # Video decoding & processing
│   │   ├── audio/          # Audio decoding & processing
│   │   ├── image/          # Image processing
│   │   ├── media.rs        # Media file management
│   │   ├── pipeline.rs     # Processing pipeline
│   │   └── errors.rs       # Error definitions
│   ├── build.rs            # Build script
│   └── Cargo.toml
├── terminal-player/        # Terminal player application
│   ├── src/
│   │   ├── renderer.rs     # ASCII renderer
│   │   ├── terminal.rs     # Terminal management
│   │   ├── player.rs       # Player controls
│   │   ├── audio.rs        # Audio playback
│   │   ├── char_maps.rs    # Character map definitions
│   │   └── main.rs         # Main application
│   └── Cargo.toml
└── downloader/             # Download functionality (experimental)
    ├── src/
    │   ├── youtube.rs      # YouTube support
    │   ├── errors.rs       # Error definitions
    │   └── lib.rs
    └── Cargo.toml
```

## System Requirements

### Dependencies

#### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y build-essential pkg-config
sudo apt install -y libavformat-dev libavcodec-dev libavutil-dev libswscale-dev libswresample-dev
sudo apt install -y libopencv-dev libclang-dev
```

#### Linux (CentOS/RHEL/Fedora)

```bash
sudo dnf install -y ffmpeg-devel opencv-devel clang-devel pkg-config
```

#### macOS (Homebrew)

```bash
brew install ffmpeg opencv pkg-config
```

#### Windows (vcpkg recommended)

```bash
vcpkg install ffmpeg opencv[core,imgproc,videoio]
```

### Optional Dependencies

For YouTube video download functionality:

```bash
pip install yt-dlp
```

Manual media download example:

```bash
yt-dlp -f best https://www.youtube.com/watch?v=SW3GGXbLDv4 -o video.mp4
```

## Installation

### From Cargo

```bash
cargo install terminal-player
```

### Build from Source

```bash
git clone https://github.com/itsakeyfut/ascii-term.git
cd ascii-term
cargo build --release
```

## Usage

### Basic Commands

```bash
# Play video file
terminal-player video.mp4

# Play audio file
terminal-player music.mp3

# Display image
terminal-player image.jpg

# YouTube URL (experimental)
terminal-player "https://www.youtube.com/watch?v=n8CojbNl2ZA"

# Loop playback
terminal-player -l video.mp4

# Custom frame rate
terminal-player --fps 24 video.mp4

# Grayscale mode
terminal-player -g video.mp4
```

### Command Line Options

```
USAGE:
    terminal-player [OPTIONS] <INPUT>

ARGS:
    <INPUT>    Input file path or URL

OPTIONS:
    -f, --fps <FPS>              Force specific frame rate
    -b, --browser <BROWSER>      Browser for cookie extraction [default: firefox]
    -l, --loop-playback          Loop playback
    -c, --char-map <CHAR_MAP>    Character map selection (0-9) [default: 0]
    -g, --gray                   Enable grayscale mode
    -w, --width-mod <WIDTH_MOD>  Width modifier for character aspect ratio [default: 1]
        --allow-frame-skip       Allow frame skipping when behind
    -n, --newlines               Add newlines to output
        --no-audio               Disable audio playback
    -h, --help                   Print help information
    -V, --version                Print version information
```

### Interactive Controls

| Key    | Function              |
| ------ | --------------------- |
| Space  | Play/Pause toggle     |
| Q, Esc | Quit player           |
| M      | Toggle mute/unmute    |
| G      | Toggle grayscale mode |
| 0-9    | Switch character maps |
| H      | Show help             |

## Library Usage

### Core Media Processing

```rust
use media_core::{MediaFile, MediaType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize library
    media_core::init()?;

    // Open media file
    let media = MediaFile::open("video.mp4")?;

    println!("Media Type: {:?}", media.media_type);
    println!("Duration: {:?}", media.info.duration);

    if media.info.has_video {
        println!("Resolution: {}x{}",
                 media.info.width.unwrap_or(0),
                 media.info.height.unwrap_or(0));
    }

    Ok(())
}
```

### Video Decoding

```rust
use media_core::{MediaFile, video::VideoDecoder};

let media = MediaFile::open("video.mp4")?;
let mut decoder = VideoDecoder::new(&media)?;

// Decode frames
while let Ok((stream, packet)) = media.format_context().read_packet() {
    if let Some(frame) = decoder.decode_next_frame(&packet)? {
        println!("Frame: {}x{} at {:?}",
                 frame.width, frame.height, frame.timestamp);
    }
}
```

### ASCII Rendering

```rust
use terminal_player::renderer::{AsciiRenderer, RenderConfig};
use image::open;

let config = RenderConfig {
    target_width: 80,
    target_height: 24,
    char_map_index: 0,
    grayscale: false,
    add_newlines: true,
    width_modifier: 1,
};

let mut renderer = AsciiRenderer::new(config);
let image = open("image.jpg")?;
let result = renderer.render_image(&image)?;

println!("{}", result.ascii_text);
```

## Technical Highlights

### Design Philosophy

- **Safety through wrapper crates**: Extensive use of FFmpeg and OpenCV Rust wrappers for memory safety and error handling
- **Modular architecture**: Clear separation between core media processing and UI presentation layers
- **Future-ready foundation**: Designed as a stepping stone toward comprehensive video editing software
- **Responsibility separation**: Media processing logic isolated in `media-core` for reusability

### Performance Characteristics

- **Memory usage**: Optimized streaming with controlled buffering
- **CPU efficiency**: Multi-threaded processing with minimal overhead
- **Real-time performance**: Frame-accurate synchronization with low latency
- **Terminal compatibility**: Adaptive rendering for various terminal sizes and capabilities

## Current Limitations & Future Development

### Known Issues

- **Audio synchronization timing**: Minor timing discrepancies in certain scenarios
- **Memory usage optimization**: Opportunities for further memory efficiency improvements
- **Image display timing**: Static image timing synchronization needs refinement

These issues are primarily concentrated in the `terminal-player` UI layer, making them good targets for focused improvement efforts.

### Experimental Features

- **YouTube download functionality**: Implemented but requires refinement for reliable operation
- **URL-triggered downloads**: Web-based media acquisition (currently unstable)

For reliable operation, we recommend preparing local media files in advance. The `yt-dlp` tool provides excellent media downloading capabilities:

```bash
# Install yt-dlp for easy media acquisition
pip install yt-dlp

# Download media for testing
yt-dlp -f best "https://www.youtube.com/watch?v=fuLPnN62p1g" -o "%(title)s.%(ext)s"
```

### Roadmap

- **Enhanced video editing capabilities**: Building upon the `media-core` foundation
- **Advanced filtering and effects**: Static image processing and video filters
- **Performance optimizations**: Memory usage and processing efficiency improvements
- **UI/UX enhancements**: Better terminal player experience
- **Streaming support**: Network media stream handling
- **Plugin architecture**: Extensible processing modules

## Troubleshooting

### Build Issues

1. **FFmpeg not found**

```bash
# Install pkg-config
sudo apt install pkg-config  # Linux
brew install pkg-config      # macOS
```

2. **OpenCV not found**

```bash
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include
```

3. **Windows vcpkg issues**

```bash
set VCPKG_ROOT=C:\vcpkg
vcpkg integrate install
```

### Runtime Issues

1. **Audio device not found**
   - Use `--no-audio` flag if audio device unavailable
   - For audio setup on Ubuntu/WSL:

```bash
# Audio setup for Ubuntu/WSL
sudo apt install pulseaudio-module-protocol-native
pulseaudio --start

# Configure PulseAudio
export PULSE_SERVER="unix:/mnt/wslg/runtime-dir/pulse/native"
echo "default-server = unix:/mnt/wslg/runtime-dir/pulse/native" > ~/.pulse/client.conf

# Verify audio setup
pactl info
pactl list short sinks

# Test audio playback
paplay /usr/share/sounds/alsa/Front_Left.wav 2>/dev/null || echo "Test sound not available"
```

2. **YouTube download failures**

   - Ensure yt-dlp is up to date
   - Try different browser options (`--browser chrome`)

3. **Terminal display issues**
   - Use larger terminal windows
   - Adjust character width with `--width-mod 2`
   - Try different character maps (0-9)

## Contributing

Contributions are welcome! Areas of particular interest:

- **Performance optimizations** in `terminal-player`
- **Enhanced character mapping algorithms**
- **Audio synchronization improvements**
- **Memory usage optimization**
- **Cross-platform compatibility enhancements**

Please see [CONTRIBUTING.md](.github/CONTRIBUTING.md) for details.

## License

MIT License

## Author

**itsakeyfut**

This project demonstrates advanced Rust multimedia programming techniques, showcasing real-time media processing, ASCII art generation, and synchronized audio playback while maintaining a clean, modular architecture suitable for future expansion into comprehensive video editing software.

---

_A technical exploration of multimedia processing in Rust, featuring FFmpeg/OpenCV integration, multi-threaded architecture, and innovative terminal-based media presentation. Designed as both a functional media player and a foundation for future video editing applications._
