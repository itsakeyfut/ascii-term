# ASCII Terminal Media Player

[![Status](https://img.shields.io/badge/status-active--development-brightgreen?style=flat-square)]()
[![Rust Version](https://img.shields.io/badge/rust-1.87+-blue.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![bad_apple](./gallery/bad_apple.gif)

![cookie_bomb_rush](./gallery/cookie_bomb_rush.gif)

![yuruyuri](./gallery/yuruyuri.gif)

See more ⇒ [Gallery](./gallery/gallery.md)

A Rust-based terminal media player that converts video files into ASCII art and plays them in real-time with synchronized audio.

## Features

- **Real-time ASCII rendering** — Converts each video frame to colored ASCII art
- **A/V sync** — PTS-based frame timing with frame skipping to stay in sync with audio
- **Multiple character maps** — 10 options from basic ASCII to Unicode block/braille/gradient characters
- **Color output** — Per-character RGB color via crossterm
- **Terminal size detection** — Automatically adapts render resolution to the terminal at startup

## Project Structure

```
ascii-term/
├── Cargo.toml              # Workspace
└── app/
    ├── codec/              # Media decode & image processing library (uses avio)
    ├── ascii-term/         # Terminal player binary
    └── downloader/         # File downloader (experimental)
```

`codec` is intentionally separated from the binary so it can serve as a foundation for future video editing software.

## Requirements

### FFmpeg

#### Linux (Ubuntu/Debian)

```bash
sudo apt install -y libavformat-dev libavcodec-dev libavutil-dev libswscale-dev libswresample-dev
sudo apt install -y libclang-dev pkg-config
```

#### Linux (Fedora/RHEL)

```bash
sudo dnf install -y ffmpeg-devel clang-devel pkg-config
```

#### macOS

```bash
brew install ffmpeg pkg-config
```

#### Windows

```bash
vcpkg install ffmpeg
set VCPKG_ROOT=C:\vcpkg
vcpkg integrate install
```

### OpenCV

#### Linux

```bash
sudo apt install -y libopencv-dev
```

#### macOS

```bash
brew install opencv
```

#### Windows

```bash
vcpkg install opencv[core,imgproc,videoio]
```

### Optional: yt-dlp (for downloading videos)

```bash
pip install yt-dlp

# Example
yt-dlp -f best "https://www.youtube.com/watch?v=SW3GGXbLDv4" -o video.mp4
```

## Installation

```bash
git clone https://github.com/itsakeyfut/ascii-term.git
cd ascii-term
cargo build --release
```

## Usage

```bash
# Play a video
ascii-term video.mp4

# Grayscale mode
ascii-term -g video.mp4

# Loop playback
ascii-term -l video.mp4

# Select character map (0–9)
ascii-term -c 6 video.mp4

# Disable audio
ascii-term --no-audio video.mp4
```

### Options

```
USAGE:
    ascii-term [OPTIONS] <INPUT>

ARGS:
    <INPUT>    Input file path

OPTIONS:
    -f, --fps <FPS>              Override frame rate
    -l, --loop-playback          Loop playback
    -c, --char-map <CHAR_MAP>    Character map index (0–9) [default: 0]
    -g, --gray                   Grayscale mode
    -w, --width-mod <WIDTH_MOD>  Width divisor for character aspect ratio [default: 1]
        --no-audio               Disable audio
    -h, --help                   Print help
    -V, --version                Print version
```

### Keyboard Controls

| Key       | Action                   |
|-----------|--------------------------|
| `Space`   | Play / Pause             |
| `Q` / `Esc` | Quit                   |
| `M`       | Toggle mute              |
| `G`       | Toggle grayscale         |
| `C`       | Cycle character map      |
| `?`       | Show help                |

### Character Maps

| Index | Name     | Characters              |
|-------|----------|-------------------------|
| 0     | Basic    | ` .:-=+*#%@`            |
| 1     | Extended | 67-char set             |
| 2     | Full     | 92-char set             |
| 3     | Blocks   | ` ░▒▓█`                 |
| 4     | Braille  | ` ⠁⠃⠇⠏⠟⠿⣿`             |
| 5     | Dots     | dot-based               |
| 6     | Gradient | ` ▁▂▃▄▅▆▇█`             |
| 7     | Binary   | black/white             |
| 8     | BinDots  | binary dots             |
| 9     | Emoji    | emoji-style             |

## Audio (WSL / Ubuntu)

If you are running inside WSL and have no audio output, configure PulseAudio:

```bash
sudo apt install -y pulseaudio
pulseaudio --start
export PULSE_SERVER="unix:/mnt/wslg/runtime-dir/pulse/native"
echo "default-server = unix:/mnt/wslg/runtime-dir/pulse/native" > ~/.pulse/client.conf
```

Use `--no-audio` to skip audio playback entirely.

## Roadmap

- Seek support
- `downloader` stabilization (URL → local file → play)
- Encode / transcode API in `codec`

## License

MIT
