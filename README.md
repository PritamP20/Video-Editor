# Video Editor CLI

A Rust-based command-line tool for basic video editing tasks, utilizing `ffmpeg` under the hood.

## Features

- **Combine**: Merge multiple video files into one.
- **Compress**: Reduce video file size using CRF (Constant Rate Factor).
- **Add Music**: Add a background audio track to a video (mixing or replacing).
- **Timelapse**: Speed up a video to create a timelapse effect.
- **Info**: Display detailed metadata about a video file.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [FFmpeg](https://ffmpeg.org/download.html) (must be added to your system PATH)

## Installation

```bash
git clone <repository-url>
cd video-editor
cargo build --release
```

## Usage

You can run the tool using `cargo run -- <command> [args]` or by running the binary directly after building.

### 1. Combine Videos
Concatenate multiple video files into a single output.

```bash
cargo run -- combine --inputs v1.mp4 v2.mp4 v3.mp4 --output combined.mp4
```

### 2. Compress Video
Compress a video to reduce file size.
- `--crf`: Constant Rate Factor (0-51). Lower is better quality, higher is lower size. Default is 23.

```bash
cargo run -- compress --input input.mp4 --output output.mp4 --crf 28
```

### 3. Add Music
Add an audio file to a video.
- `--reduce-original`: Volume of the original video audio (e.g., `0.1` for 10%). Default is `1.0`.

```bash
cargo run -- add-music --video input.mp4 --audio music.mp3 --output output.mp4 --reduce-original 0.2
```

### 4. Create Timelapse
Speed up a video. usage of `--speed` factor. Audio is removed.

```bash
cargo run -- timelapse --input input.mp4 --output output.mp4 --speed 10.0
```

### 5. Get Video Info
Show metadata about a video file (resolution, codecs, bitrate, etc.).

```bash
cargo run -- info --input input.mp4
```

## License

MIT
