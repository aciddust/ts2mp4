# TS to MP4 Converter

A Rust library that converts MPEG-2 TS (Transport Stream) files to MP4 format. Can be compiled to WebAssembly for use in web browsers.

## Before You Start

This project started as a personal implementation to understand TS (Transport Stream) and MP4 formats,
and for this reason, the content may be somewhat rough around the edges.

## Features

- **H.264 video** + **AAC audio** full support
- **Thumbnail extraction** from TS and MP4 files
- Pure Rust implementation (no SharedArrayBuffer required)
- WebAssembly support
- Safe for web environments with single-threaded operation
- Zero-copy optimization
- Compatible with major players like QuickTime, VLC, ffplay

## Usage

### As a Rust Library

If you wanna specific version, add to your Cargo.toml:

```toml
[dependencies]
ts2mp4 = "0.1.0"
```

then...

```rust
use ts2mp4::{convert_ts_to_mp4, extract_thumbnail_from_ts, extract_thumbnail_from_mp4};
use std::fs;

fn main() -> std::io::Result<()> {
    // Convert TS to MP4
    let ts_data = fs::read("input.ts")?;
    let mp4_data = convert_ts_to_mp4(&ts_data)?;
    fs::write("output.mp4", mp4_data)?;

    // Extract thumbnail from TS
    let thumbnail_data = extract_thumbnail_from_ts(&ts_data)?;
    fs::write("thumbnail_ts.h264", thumbnail_data)?;

    // Extract thumbnail from MP4
    let mp4_data = fs::read("input.mp4")?;
    let thumbnail_data = extract_thumbnail_from_mp4(&mp4_data)?;
    fs::write("thumbnail_mp4.h264", thumbnail_data)?;

    Ok(())
}
```

### As a CLI Tool

```bash
cargo install ts2mp4

# Convert TS to MP4
cargo run --release -- convert input.ts output.mp4

# Extract thumbnail from TS file
cargo run --release -- thumbnail-ts input.ts thumbnail.h264

# Extract thumbnail from MP4 file
cargo run --release -- thumbnail-mp4 input.mp4 thumbnail.h264
```

The thumbnail is extracted as a raw H.264 keyframe (I-frame) which can be:

- Converted to an image using ffmpeg: `ffmpeg -i thumbnail.h264 -frames:v 1 thumbnail.jpg`
- Used directly in video processing applications
- Decoded by H.264 decoders

### For developer

```bash
# dev
cargo build

# release
cargo build --release

# then
./target/release/ts2mp4 ${INPUT_TS} ${OUTPUT_MP4}
```

## For WebAssembly

Build from source:

```bash
# Install wasm-pack
cargo install wasm-pack

# Build
wasm-pack build --target web
```

This will generate the following files in the `pkg/` directory:

- `ts2mp4.js` - JavaScript bindings
- `ts2mp4_bg.wasm` - WebAssembly binary
- `ts2mp4.d.ts` - TypeScript type definitions

### Using in Web Browsers

[DEMO Page](https://aciddust.github.io/ts2mp4)

Examples:

- [Convert TS to MP4](./web/index.html)
- [Extract Thumbnail](./web/thumbnail.html)

```javascript
import init, {
  convert_ts_to_mp4_wasm,
  extract_thumbnail_from_ts_wasm,
  extract_thumbnail_from_mp4_wasm
} from './pkg/ts2mp4.js';

// Initialize WASM
await init();

// Convert TS to MP4
const tsData = new Uint8Array(await file.arrayBuffer());
const mp4Data = convert_ts_to_mp4_wasm(tsData);

// Extract thumbnail from TS
const thumbnailFromTs = extract_thumbnail_from_ts_wasm(tsData);

// Extract thumbnail from MP4
const mp4Data = new Uint8Array(await mp4File.arrayBuffer());
const thumbnailFromMp4 = extract_thumbnail_from_mp4_wasm(mp4Data);
```

## Why Not Use SharedArrayBuffer?

This library is intentionally designed not to use SharedArrayBuffer:

### Advantages

1. **Browser Compatibility**: SharedArrayBuffer requires COOP/COEP header configuration, making it difficult to use in many hosting environments
2. **Security**: Restricted in many browsers to mitigate Spectre vulnerabilities
3. **Simplicity**: Can be used immediately without complex server configuration
4. **Single-threaded**: Simple and predictable memory management

### Performance Optimization Techniques

Good performance can be achieved without SharedArrayBuffer:

1. **Streaming Processing**: Process in chunks instead of loading entire file into memory
2. **Web Workers**: Execute in workers to prevent main thread blocking
3. **Asynchronous Processing**: Split work for large files

```javascript
// Example using Web Worker
// worker.js
import init, { convert_ts_to_mp4_wasm } from './pkg/ts2mp4.js';

self.onmessage = async (e) => {
    await init();

    try {
        const mp4Data = convert_ts_to_mp4_wasm(e.data);
        self.postMessage({ success: true, data: mp4Data });
    } catch (error) {
        self.postMessage({ success: false, error: error.message });
    }
};

// main.js
const worker = new Worker('worker.js', { type: 'module' });

worker.onmessage = (e) => {
    if (e.data.success) {
        // Process MP4 data
        const blob = new Blob([e.data.data], { type: 'video/mp4' });
        // ...
    } else {
        console.error('Conversion failed:', e.data.error);
    }
};

// Start conversion
worker.postMessage(tsData);
```

## Supported Features

### Currently Supported

- **Video Codec**: H.264 (AVC) Main/High Profile
- **Audio Codec**: AAC-LC (Stereo, 48kHz)
- **Container**: MP4 (ISO/IEC 14496-12 compliant)
- **Metadata**: Complete moov/trak/stbl structure
- **Timestamps**: Accurate synchronization based on PTS/DTS
- **Auto-detection**:
  - Resolution (SPS parsing)
  - Frame rate

### Future Plans

- Additional audio codecs (Opus, etc.)
- Multiple audio/subtitle tracks
- Variable frame rate support
- HDR metadata
- Timestamp processing improvements

## Development Guide

For detailed development and debugging instructions, see [DEV_GUIDE.md](DEV_GUIDE.md).

### Quick Start

```bash
# Build
cargo build --release

# Test
./target/release/ts2mp4 input.ts output.mp4

# Verify
ffprobe output.mp4
ffplay output.mp4
```

### Debugging Tools

The `test-scripts/` directory contains Python scripts for analyzing MP4 file structure:

- `test-scripts/analyze_mp4.py` - Analyze box structure
- `test-scripts/check_all_durations.py` - Verify durations
- `test-scripts/check_stts.py` - Check Time-to-Sample
- `test-scripts/verify_audio_data.py` - Verify audio data positions

For detailed usage, refer to [DEV_GUIDE.md](docs/DEV_GUIDE.md).

## Troubleshooting

For common issues and solutions, refer to the "Troubleshooting" section in [DEV_GUIDE.md](docs/DEV_GUIDE.md).

### Additional Resources

| docs | description | lang |
| - | - | - |
| TS_STRUCTURE.md | TS packet generation and processing | [ko](./docs/ko/TS_STRUCTURE.md), [en](./docs/en/TS_STRUCTURE.md) |
| MP4_STRUCTURE.md | MP4 generation and structure | [ko](./docs/ko/MP4_STRUCTURE.md), [en](./docs/en/MP4_STRUCTURE.md) |
| 90kHZ_MAGIC.md | Why is 90kHz used for frame synchronization? | [ko](./docs/ko/90kHz_MAGIC.md), [en](./docs/en/90kHz_MAGIC.md) |

## License

MIT

## Contributing

Issues and PRs are always welcome!
