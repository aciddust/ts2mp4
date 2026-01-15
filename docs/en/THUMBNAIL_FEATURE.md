# Thumbnail Extraction Feature

## Overview

The ts2mp4 library now includes functionality to extract thumbnails from TS and MP4 files.

## Key Features

### Implementation

1. **TS File Thumbnail Extraction**
   - Extracts the first I-frame (IDR frame)
   - Includes SPS/PPS NAL units
   - Outputs in Annex B format

2. **MP4 File Thumbnail Extraction**
   - Extracts the first keyframe
   - Converts AVCC format to Annex B
   - Parses MP4 box structure

3. **No External Dependencies**
   - No image crate required
   - Pure Rust implementation
   - Maintains existing dependencies

### Output Format

- Raw H.264 NAL units (Annex B format)
- Can be converted to JPEG/PNG using ffmpeg
- Can be used directly by H.264 decoders

## Usage

### CLI

```bash
# Extract thumbnail from TS file
ts2mp4 thumbnail-ts input.ts thumbnail.h264

# Extract thumbnail from MP4 file
ts2mp4 thumbnail-mp4 input.mp4 thumbnail.h264

# Convert to image
ffmpeg -i thumbnail.h264 -frames:v 1 thumbnail.jpg
```

### Rust API

```rust
use ts2mp4::{extract_thumbnail_from_ts, extract_thumbnail_from_mp4};
use std::fs;

// Extract from TS
let ts_data = fs::read("input.ts")?;
let thumbnail = extract_thumbnail_from_ts(&ts_data)?;
fs::write("thumbnail.h264", thumbnail)?;

// Extract from MP4
let mp4_data = fs::read("input.mp4")?;
let thumbnail = extract_thumbnail_from_mp4(&mp4_data)?;
fs::write("thumbnail.h264", thumbnail)?;
```

### WebAssembly

```javascript
import init, {
  extract_thumbnail_from_ts_wasm,
  extract_thumbnail_from_mp4_wasm
} from './pkg/ts2mp4.js';

await init();

// Extract from TS
const tsData = new Uint8Array(await tsFile.arrayBuffer());
const thumbnail = extract_thumbnail_from_ts_wasm(tsData);

// Extract from MP4
const mp4Data = new Uint8Array(await mp4File.arrayBuffer());
const thumbnail = extract_thumbnail_from_mp4_wasm(mp4Data);
```

## Technical Details

### TS Parsing

- NAL unit search (0x00 0x00 0x00 0x01 or 0x00 0x00 0x01)
- IDR frame detection (NAL type 5)
- Complete frame composition with SPS/PPS

### MP4 Parsing

- Extract first sample from mdat box
- Check sample size from stsz box
- Convert AVCC → Annex B (length prefix → start code)

### Conversion Process

```bash
AVCC format:
[4-byte length][NAL unit][4-byte length][NAL unit]...

Annex B format:
[0x00 0x00 0x00 0x01][NAL unit][0x00 0x00 0x00 0x01][NAL unit]...
```

## File Structure

```bash
src/
  thumbnail.rs           # Thumbnail extraction module
  lib.rs                # Public API exposure
  main.rs               # CLI command implementation

examples/
  extract_thumbnail.rs  # Usage example

web/
  thumbnail.html        # Web demo page

docs/
  ko/
    USAGE.md           # Korean usage guide
    DEV_GUIDE.md       # Korean development guide
  en/
    USAGE.md           # English usage guide
    DEV_GUIDE.md       # English development guide
```

## Testing

```bash
# Unit tests
cargo test

# Release build
cargo build --release

# WASM build
wasm-pack build --target web

# Run example
cargo run --example extract_thumbnail -- input.ts thumbnail.h264
```

## Use Cases

### 1. Video Thumbnail Generation

```bash
ts2mp4 thumbnail-ts video.ts thumb.h264
ffmpeg -i thumb.h264 -vf scale=320:240 thumb_small.jpg
```

### 2. Web Applications

- Extract thumbnails directly in the browser
- Client-side processing without server upload
- Preview generation

### 3. Video Processing Pipelines

- Automatic thumbnail generation
- Video indexing
- Preview image creation

## Performance

- **Memory**: Minimal additional memory allocation
- **Speed**: Fast processing by extracting only the first I-frame
- **Size**: No binary size increase due to no external dependencies

## Limitations

- Only extracts the first keyframe (no multi-thumbnail support)
- H.264 codec only
- Image decoding requires separate tools (e.g., ffmpeg)
