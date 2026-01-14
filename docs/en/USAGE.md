# Usage Guide

## How to Run

```bash
# Build
cargo build --release

# Run conversion
./target/release/ts2mp4 input.ts output.mp4

# Verify
ffprobe output.mp4
```

## Playback Testing

The generated MP4 file's video and audio information has been verified on the following players:

- **QuickTime Player** (macOS default player)
- **VLC Media Player**
- **FFplay** (ffmpeg)
- Most web browsers (HTML5 video)
- Windows Media Player
- Mobile players (iOS Safari, Android Chrome, etc.)

**Note**:

- This converter was developed with QuickTime compatibility as the top priority.

## Main Work Summary

### Completed Features

- [x] ~~Audio track support (AAC)~~
- [x] ~~Video/audio synchronization~~
- [x] ~~QuickTime compatibility~~

### Efforts to Comply with MP4 Standard

- Video Support
  - ftyp (File Type)
  - moov (Movie Metadata)
    - mvhd (Movie Header)
    - trak (Track)
      - tkhd (Track Header)
      - mdia (Media)
        - mdhd (Media Header)
        - hdlr (Handler)
        - minf (Media Information)
          - vmhd (Video Media Header)
          - dinf (Data Information)
          - stbl (Sample Table)
            - stsd (Sample Description with avcC)
- Audio Support
  - AAC frame extraction and muxing
  - smhd (Sound Media Header)
  - esds box (AudioSpecificConfig)
    - AAC-LC profile
    - 48kHz sample rate
    - Stereo channel configuration
- Video/Audio Synchronization (PTS-based)

### Metadata Extraction

- SPS (Sequence Parameter Set) parsing
- PPS (Picture Parameter Set) extraction
- Automatic resolution detection
- avcC box generation (H.264 decoder configuration)

### Sample Table Compatibility

- Record each frame size (STSZ)
- Accurate chunk offset calculation (STCO)
- Include timing information (STTS)
- Single chunk optimization (compatibility)

### Timescale Unification

- Use 90kHz timescale for all tracks
- Video: sample_delta = 3000 (30fps)
- Audio: sample_delta = 1920 (AAC 1024 samples @ 48kHz)
- Developed based on QuickTime Player execution
- avcC box generation (decoder configuration)

## Debugging and Verification

For detailed verification methods, see [DEV_GUIDE.md](DEV_GUIDE.md).

### Quick Verification

```bash
# Basic information
ffprobe output.mp4

# Playback test
ffplay output.mp4
```

### Expected Output Example

```bash
Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'output2.mp4':
  Duration: 00:00:23.36
  Stream #0:0: Video: h264 (Main), yuv420p, 1280x720, 3203 kb/s, 30 fps
  Stream #0:1: Audio: aac (LC), 48000 Hz, stereo, fltp, 192 kb/s
```

### Limitations

Current version limitations:

1. **Video Only Processing**: Audio tracks not yet supported
2. **H.264 Only**: MPEG-2 and other codecs not supported
3. **Simple SPS Parsing**: Complex profiles use default values
4. **Fixed Frame Rate**: Variable frame rate not supported
