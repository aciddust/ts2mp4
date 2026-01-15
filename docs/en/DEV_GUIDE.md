# Development Guide

Development and debugging guide for TS to MP4 converter.

## Build Instructions

### Debug Build

```bash
cargo build
```

### Release Build (Optimized)

```bash
cargo build --release
```

Executable location:

- Debug: `target/debug/ts2mp4`
- Release: `target/release/ts2mp4`

### WebAssembly Build

```bash
# Install wasm-pack (first time only)
cargo install wasm-pack

# Build WASM
wasm-pack build --target web
```

## Usage

### Basic Usage

```bash
# Convert TS to MP4
./target/release/ts2mp4 convert <input.ts> <output.mp4>

# Extract thumbnail from TS file
./target/release/ts2mp4 thumbnail-ts <input.ts> <output.h264>

# Extract thumbnail from MP4 file
./target/release/ts2mp4 thumbnail-mp4 <input.mp4> <output.h264>

# Examples
./target/release/ts2mp4 convert input.ts output.mp4
./target/release/ts2mp4 thumbnail-ts input.ts thumbnail.h264
./target/release/ts2mp4 thumbnail-mp4 output.mp4 thumbnail.h264

# Convert thumbnail to image
ffmpeg -i thumbnail.h264 -frames:v 1 thumbnail.jpg
```

### Output Example

```bash
Converting input.ts to output.mp4
Found PIDs - Video: 256, Audio: 257
Total audio frames collected: 1095
Total video frames collected: 700
Audio PTS range: 104160 - 2179680 (1.16 - 24.22 sec)
Video PTS range: 108000 - 2205000 (1.20 - 24.50 sec)
Conversion completed successfully!
```

## Testing and Verification

### Check Metadata with ffprobe

```bash
# Basic information
ffprobe output.mp4

# Stream information only
ffprobe -v error -show_streams output.mp4

# Check duration only
ffprobe -v error -show_entries stream=codec_name,duration -of default=nw=1 output.mp4

# Detailed format information
ffprobe -v error -show_format -show_streams output.mp4
```

**Expected Output:**

```bash
Stream #0:0(und): Video: h264 (Main) (avc1 / 0x31637661), yuv420p(progressive),
  1280x720 [SAR 1:1 DAR 16:9], 3203 kb/s, 30 fps, 30 tbr, 90k tbn (default)
Stream #0:1(und): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp,
  192 kb/s (default)
```

### Playback Test with ffplay

```bash
# Basic playback
ffplay output.mp4

# Auto-exit (after playback completes)
ffplay -autoexit output.mp4

# Error output only
ffplay -autoexit -loglevel error output.mp4
```

### Check Packet Information

```bash
# Show all packets
ffprobe -v error -show_packets output.mp4

# Audio packets only
ffprobe -v error -select_streams a:0 -show_packets output.mp4

# Video packets only
ffprobe -v error -select_streams v:0 -show_packets output.mp4

# Check packet count
ffprobe -v error -count_packets -show_entries stream=nb_read_packets output.mp4
```

## Debugging Tools (Python Scripts)

The project includes several Python scripts for analyzing MP4 structure.

### 1. analyze_mp4.py - MP4 Structure Analysis

**Purpose**: Check basic box structure of MP4 file

```bash
python3 test-scripts/analyze_mp4.py output.mp4
```

**Output Information:**

- STSC (Sample-to-Chunk): Chunk structure
- STCO (Chunk Offset): Data location
- STSZ (Sample Size): Size of each sample
- MDAT: Actual media data location

**When to Use:**

- Verify chunk structure is correct
- Validate file size and offsets
- Check sample count

### 2. check_all_durations.py - Duration Verification

**Purpose**: Check all duration-related boxes

```bash
python3 test-scripts/check_all_durations.py output.mp4
```

**Output Information:**

- MVHD (Movie Header): Overall video duration
- TKHD (Track Header): Each track's duration
- MDHD (Media Header): Media-specific timescale and duration

**When to Use:**

- When playback time is incorrect in QuickTime
- Diagnose duration mismatch issues
- Verify timescale settings

**Expected Output:**

```bash
MVHD (Movie Header):
  Timescale: 90000 Hz
  Duration: 2100000 units
  Duration: 23.333 seconds

TKHD (Track Headers):
  Video Track:
    Duration: 2100000 (in movie timescale 90000 Hz)
    Duration: 23.333 seconds
  Audio Track:
    Duration: 2102400 (in movie timescale 90000 Hz)
    Duration: 23.360 seconds
```

### 3. check_stts.py - Time-to-Sample Verification

**Purpose**: Check STTS box sample_delta

```bash
python3 test-scripts/check_stts.py output.mp4
```

**Output Information:**

- Each track's sample_count and sample_delta
- Calculated total duration
- AAC frame duration verification

**When to Use:**

- When audio cuts off in the middle
- When duration calculation is wrong
- Verify timescale conversion

**Note:**

- Video sample_delta: 3000 (30fps, 90kHz)
- Audio sample_delta: 1920 (1024 samples @ 48kHz in 90kHz)

### 4. verify_audio_data.py - Audio Data Position Verification

**Purpose**: Verify audio data positions in actual file

```bash
python3 test-scripts/verify_audio_data.py output.mp4
```

**When to Use:**

- When audio cuts off at a specific point
- Verify data actually exists in file
- Validate STCO offsets are correct

### 5. debug_n_sec.py - Specific Time Position Debugging

**Purpose**: Detailed analysis at a specific time position (default 9 seconds, customizable)

```bash
python3 test-scripts/debug_n_sec.py output.mp4
python3 test-scripts/debug_n_sec.py output.mp4 --time 12.5
```

**When to Use:**

- When playback stops at a specific time
- Verify data offset calculations

## Understanding MP4 Structure

### Basic Box Structure

```bash
MP4 File
├── ftyp (File Type)
├── moov (Movie Metadata)
│   ├── mvhd (Movie Header)
│   ├── trak (Video Track)
│   │   ├── tkhd (Track Header)
│   │   └── mdia (Media)
│   │       ├── mdhd (Media Header)
│   │       ├── hdlr (Handler)
│   │       └── minf (Media Information)
│   │           ├── vmhd (Video Media Header)
│   │           ├── dinf (Data Information)
│   │           └── stbl (Sample Table)
│   │               ├── stsd (Sample Description - avc1 + avcC)
│   │               ├── stts (Time-to-Sample)
│   │               ├── stsc (Sample-to-Chunk)
│   │               ├── stsz (Sample Sizes)
│   │               ├── stco (Chunk Offsets)
│   │               └── ctts (Composition Offsets - optional)
│   └── trak (Audio Track)
│       ├── tkhd (Track Header)
│       └── mdia (Media)
│           ├── mdhd (Media Header)
│           ├── hdlr (Handler)
│           └── minf (Media Information)
│               ├── smhd (Sound Media Header)
│               ├── dinf (Data Information)
│               └── stbl (Sample Table)
│                   ├── stsd (Sample Description - mp4a + esds)
│                   ├── stts (Time-to-Sample)
│                   ├── stsc (Sample-to-Chunk)
│                   ├── stsz (Sample Sizes)
│                   └── stco (Chunk Offsets)
└── mdat (Media Data)
    ├── [Video frames...]
    └── [Audio frames...]
```

### Important Timescale Concept

All durations and timestamps use **90kHz timescale**:

- **Movie timescale (mvhd)**: 90000 Hz
- **Video media timescale (mdhd)**: 90000 Hz
- **Audio media timescale (mdhd)**: 90000 Hz (not 48000!)

**Calculation Examples:**

- Video 1 frame (30fps): 90000 / 30 = 3000 units
- Audio 1 frame (AAC 1024 samples @ 48kHz): 1024 / 48000 * 90000 = 1920 units

### STTS (Time-to-Sample) Settings

```rust
// Video
sample_count: 700
sample_delta: 3000  // 90000 / 30fps

// Audio
sample_count: 1095
sample_delta: 1920  // (1024 samples / 48000 Hz) * 90000
```

### STSC (Sample-to-Chunk) Optimization

Using single chunk approach (improved compatibility):

```rust
// Video
first_chunk: 1
samples_per_chunk: 700  // All samples in 1 chunk

// Audio
first_chunk: 1
samples_per_chunk: 1095  // All samples in 1 chunk
```

**Reason**: Creating many small chunks causes playback issues in some players like QuickTime

## Common Troubleshooting

### 1. Audio Cuts Off in the Middle

**Symptoms**: ffplay works fine, QuickTime stops at 12 seconds

**Cause**: STTS sample_delta or MDHD timescale mismatch

**Solution**:

```bash
# 1. Check durations
python3 test-scripts/check_all_durations.py output.mp4

# 2. Check STTS
python3 test-scripts/check_stts.py output.mp4

# 3. Verify audio sample_delta is 1920
# 4. Verify audio mdhd timescale is 90000
```

### 2. Playback Time Differs from Actual

**Symptoms**: ffprobe shows 23 seconds but actual playback is 12 seconds

**Cause**: TKHD duration calculated with wrong timescale

**Solution**:

- TKHD duration must use **movie timescale (90kHz)**
- MDHD duration must use **media timescale (90kHz)**
- Both should have the same value

### 3. Won't Play in QuickTime

**Symptoms**: Works in ffplay but not in QuickTime

**Possible Causes**:

1. Missing or incorrect avcC box
2. Missing esds box (audio)
3. Duration mismatch
4. Box size errors

**Solution**:

```bash
# 1. Check structure
python3 test-scripts/analyze_mp4.py output.mp4

# 2. Verify codecs with ffprobe
ffprobe -v error -show_streams output.mp4

# 3. Manually check box sizes
xxd output.mp4 | head -100
```

### 4. Audio Sync Issues

**Symptoms**: Audio/video out of sync

**Cause**: Global minimum PTS normalization issue

**Solution**:

- Check `global_min_pts` calculation in code
- Consider adding Edit List (edts) box
- If audio starts later than video, adjust with edts

## Code Modification Checklist

When modifying box structure, verify:

- [ ] Is the box size field accurate?
- [ ] Is timescale consistent? (all 90kHz)
- [ ] Is sample_delta calculation correct?
- [ ] Does duration match in all headers?
- [ ] Do STCO offsets match actual data positions?
- [ ] Are all sample sizes recorded in STSZ?
- [ ] Does STSC match actual chunk structure?

## Direct Verification with hexdump

```bash
# Check MP4 header (first 100 lines)
xxd output.mp4 | head -100

# Find specific boxes
xxd output.mp4 | grep "mvhd"
xxd output.mp4 | grep "stts"

# Find mdat position
xxd output.mp4 | grep -n "mdat"
```

## Additional References

- [ISO/IEC 14496-12:2022](https://www.iso.org/standard/83102.html) - ISO base media file format
- [ISO/IEC 14496-14](https://www.iso.org/standard/79110.html) - MP4 file format
- [ISO/IEC 14496-15:2024](https://www.iso.org/standard/89118.html) - AVC file format (NAL unit structure)
- [MP4RA](https://mp4ra.org/) - MP4 Registration Authority

## Debugging Tips

1. **Always run ffprobe first**: Verify basic structure is correct
2. **Use Python scripts**: Check detailed information per box
3. **Test with ffplay**: Verify actual playback capability
4. **Final validation with QuickTime**: Most strict player
5. **Step-by-step debugging**: Video only first → Add audio → Synchronization

## Performance Optimization

- Use Release builds (`--release`)
- Consider chunk-based processing for large files
- Recommended to use Web Workers in WASM
- Minimize unnecessary box creation
