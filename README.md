# TS to MP4 Converter

A Rust library that converts MPEG-2 TS (Transport Stream) files to MP4 format. Can be compiled to WebAssembly for use in web browsers.

## Before You Start

This project started as a personal implementation to understand TS (Transport Stream) and MP4 formats,
and for this reason, the content may be somewhat rough around the edges.

### Additional Resources

| docs | description | lang |
| - | - | - |
| TS_STRUCTURE.md | TS packet generation and processing | [ko](./docs/ko/TS_STRUCTURE.md), [en](./docs/en/TS_STRUCTURE.md) |
| MP4_STRUCTURE.md | MP4 generation and structure | [ko](./docs/ko/MP4_STRUCTURE.md), [en](./docs/en/MP4_STRUCTURE.md) |
| 90kHZ_MAGIC.md | Why is 90kHz used for frame synchronization? | [ko](./docs/ko/90kHz_MAGIC.md), [en](./docs/en/90kHz_MAGIC.md) |

## Features

- **H.264 video** + **AAC audio** full support
- Pure Rust implementation (no SharedArrayBuffer required)
- WebAssembly support
- Safe for web environments with single-threaded operation
- Zero-copy optimization
- Compatible with major players like QuickTime, VLC, ffplay

## Installation

```bash
cargo build --release
```

## Usage

### CLI Usage

```bash
cargo run --release -- input.ts output.mp4
```

### Using as a Rust Library

```rust
use ts2mp4::convert_ts_to_mp4;
use std::fs;

fn main() -> std::io::Result<()> {
    let ts_data = fs::read("input.ts")?;
    let mp4_data = convert_ts_to_mp4(&ts_data)?;
    fs::write("output.mp4", mp4_data)?;
    Ok(())
}
```

## WebAssembly Build

### Prerequisites

```bash
# Install wasm-pack
cargo install wasm-pack

# Or add wasm32 target
rustup target add wasm32-unknown-unknown
```

### How to Build WASM

#### 1. Using wasm-pack (Recommended)

```bash
wasm-pack build --target web
```

This will generate the following files in the `pkg/` directory:

- `ts2mp4.js` - JavaScript bindings
- `ts2mp4_bg.wasm` - WebAssembly binary
- `ts2mp4.d.ts` - TypeScript type definitions

#### 2. Manual Build

```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ts2mp4.wasm --out-dir pkg --target web
```

### Using in Web Browsers

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>TS to MP4 Converter</title>
</head>
<body>
    <input type="file" id="fileInput" accept=".ts">
    <button id="convertBtn">Convert to MP4</button>

    <script type="module">
        import init, { convert_ts_to_mp4_wasm } from './pkg/ts2mp4.js';

        async function convertFile() {
            // Initialize WASM
            await init();

            const fileInput = document.getElementById('fileInput');
            const file = fileInput.files[0];

            if (!file) {
                alert('Please select a file');
                return;
            }

            // Read file
            const arrayBuffer = await file.arrayBuffer();
            const tsData = new Uint8Array(arrayBuffer);

            try {
                // Convert TS to MP4 (no SharedArrayBuffer required)
                const mp4Data = convert_ts_to_mp4_wasm(tsData);

                // Download MP4 file
                const blob = new Blob([mp4Data], { type: 'video/mp4' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = 'output.mp4';
                a.click();
                URL.revokeObjectURL(url);

                alert('Conversion successful!');
            } catch (error) {
                console.error('Conversion failed:', error);
                alert('Conversion failed: ' + error);
            }
        }

        document.getElementById('convertBtn').addEventListener('click', convertFile);
    </script>
</body>
</html>
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

## License

MIT

## Contributing

Issues and PRs are always welcome!
