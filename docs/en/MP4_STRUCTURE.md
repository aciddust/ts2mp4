# MP4 File Structure

MP4 (MPEG-4 Part 14) is a container format based on the ISO Base Media File Format. It can store video, audio, subtitles, and more.

## 1. Basic Concepts

### 1.1 Box (Atom) Structure

MP4 is composed of basic units called "Boxes". Each Box has the following structure:

```bash
[4 bytes] Size - Total box size (including header)
[4 bytes] Type - Box type (FourCC)
[Size-8 bytes] Data - Box data (may contain nested boxes)
```

**Special Cases:**

- Size = 0: Extends to end of file
- Size = 1: Next 8 bytes contain 64-bit size

**Creating a Box in Code:**

```rust
// Example: Creating stts box
let stts_size = 8 + stts.len();
let mut stts_box = Vec::new();
stts_box.extend_from_slice(&(stts_size as u32).to_be_bytes());   // Size
stts_box.extend_from_slice(b"stts");                             // Type
stts_box.extend_from_slice(&stts);                               // Data
```

### 1.2 Overall File Structure

```bash
MP4 File
├─ ftyp (File Type Box)
├─ moov (Movie Box) - Metadata
│   ├─ mvhd (Movie Header)
│   ├─ trak (Track) - Video
│   │   ├─ tkhd (Track Header)
│   │   └─ mdia (Media)
│   │       ├─ mdhd (Media Header)
│   │       ├─ hdlr (Handler)
│   │       └─ minf (Media Information)
│   │           ├─ vmhd (Video Media Header)
│   │           ├─ dinf (Data Information)
│   │           └─ stbl (Sample Table)
│   │               ├─ stsd (Sample Description)
│   │               ├─ stts (Time-to-Sample)
│   │               ├─ stsc (Sample-to-Chunk)
│   │               ├─ stsz (Sample Sizes)
│   │               ├─ stco (Chunk Offsets)
│   │               └─ ctts (Composition Time-to-Sample)
│   └─ trak (Track) - Audio
│       └─ ... (similar to video)
└─ mdat (Media Data Box) - Actual data
    ├─ [Video frames...]
    └─ [Audio frames...]
```

## 2. Detailed Box Descriptions

### 2.1 ftyp (File Type Box)

Contains brand and compatibility information.

**Structure:**

```bash
Size (4 bytes)
Type: 'ftyp'
Major Brand (4 bytes) - Primary brand
Minor Version (4 bytes) - Version
Compatible Brands (4 bytes × N) - List of compatible brands
```

**Code Generation:**

```rust
mp4_buffer.extend_from_slice(&[
    0x00, 0x00, 0x00, 0x1C,  // Size: 28 bytes
    b'f', b't', b'y', b'p',  // Type: ftyp
    b'i', b's', b'o', b'm',  // Major brand: isom (ISO Base Media)
    0x00, 0x00, 0x02, 0x00,  // Minor version: 512
    b'i', b's', b'o', b'm',  // Compatible: isom
    b'i', b's', b'o', b'2',  // Compatible: iso2
    b'm', b'p', b'4', b'1',  // Compatible: mp41
]);
```

**Common Brands:**

- `isom`: ISO Base Media File Format
- `iso2`: ISO Base Media File Format version 2
- `mp41`: MPEG-4 version 1
- `avc1`: H.264/AVC

### 2.2 moov (Movie Box)

Container box that holds all metadata.

**Characteristics:**

- Can be positioned at the beginning or end of file
- When at beginning: fast start playback
- Contains information for all tracks

### 2.3 mvhd (Movie Header Box)

Defines properties of the entire video.

**Structure (Version 0, 108 bytes):**

```bash
Size (4 bytes)
Type: 'mvhd'
Version (1 byte) - 0
Flags (3 bytes) - 0

Creation Time (4 bytes) - Seconds since January 1, 1904
Modification Time (4 bytes)
Timescale (4 bytes) - Units representing 1 second (e.g., 90000)
Duration (4 bytes) - Playback duration in timescale units

Preferred Rate (4 bytes) - Playback rate (1.0 = 0x00010000)
Preferred Volume (2 bytes) - Volume (1.0 = 0x0100)
Reserved (10 bytes)
Matrix (36 bytes) - Video transformation matrix
Pre-defined (24 bytes)
Next Track ID (4 bytes) - Next track ID
```

**Code Generation:**

```rust
fn build_mvhd(duration: u32, has_audio: bool) -> Vec<u8> {
    let next_track_id = if has_audio { 3 } else { 2 };

    vec![
        0x00, 0x00, 0x00, 0x6C,  // Size: 108 bytes
        b'm', b'v', b'h', b'd',  // Type
        0x00,                     // Version 0
        0x00, 0x00, 0x00,        // Flags

        0x00, 0x00, 0x00, 0x00,  // Creation time
        0x00, 0x00, 0x00, 0x00,  // Modification time

        0x00, 0x01, 0x5F, 0x90,  // Timescale: 90000 Hz
        // Duration (4 bytes) - calculated
        duration.to_be_bytes()[0], duration.to_be_bytes()[1],
        duration.to_be_bytes()[2], duration.to_be_bytes()[3],

        0x00, 0x01, 0x00, 0x00,  // Preferred rate: 1.0
        0x01, 0x00,              // Preferred volume: 1.0

        // Reserved (10 bytes)
        0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,

        // Matrix (36 bytes) - Identity matrix
        0x00, 0x01, 0x00, 0x00,  // [1.0, 0, 0]
        // ... (remaining matrix values)

        // Pre-defined (24 bytes)
        0x00, 0x00, 0x00, 0x00,
        // ...

        // Next track ID
        (next_track_id >> 24) as u8,
        (next_track_id >> 16) as u8,
        (next_track_id >> 8) as u8,
        next_track_id as u8,
    ]
}
```

**Duration Calculation:**

```bash
Video: frame_count × 3000  (30fps, 90kHz timescale)
       = frame_count × (90000 / 30)

Example: 700 frames → 700 × 3000 = 2,100,000
   → 2,100,000 / 90000 = 23.33 seconds
```

### 2.4 trak (Track Box)

Represents each media track (video, audio, etc.).

**Multiple traks can exist in one MP4:**

- Track 1: Video
- Track 2: Audio
- Track 3: Subtitles, etc.

### 2.5 tkhd (Track Header Box)

Defines properties of individual tracks.

**Structure (Version 0):**

```bash
Size (4 bytes)
Type: 'tkhd'
Version (1 byte) - 0
Flags (3 bytes) - 0x000007 (enabled, in movie, in preview)

Creation Time (4 bytes)
Modification Time (4 bytes)
Track ID (4 bytes) - Track identifier (starts from 1)
Reserved (4 bytes)
Duration (4 bytes) - In movie timescale units

Reserved (8 bytes)
Layer (2 bytes) - Video layer (0)
Alternate Group (2 bytes) - Alternate group
Volume (2 bytes) - Audio volume (0 for video)
Reserved (2 bytes)
Matrix (36 bytes) - Transformation matrix
Width (4 bytes) - Fixed point 16.16
Height (4 bytes) - Fixed point 16.16
```

**Code Generation:**

```rust
fn build_tkhd(track_id: u32, sample_count: usize, width: u16, height: u16) -> Vec<u8> {
    let duration = sample_count as u32 * 3000;  // 90kHz timescale
    let width_fixed = (width as u32) << 16;     // 16.16 fixed point
    let height_fixed = (height as u32) << 16;

    vec![
        0x00, 0x00, 0x00, 0x5C,  // Size: 92 bytes
        b't', b'k', b'h', b'd',  // Type
        0x00,                     // Version
        0x00, 0x00, 0x07,        // Flags: enabled + in movie + in preview

        // ... timestamps ...

        // Track ID
        (track_id >> 24) as u8, (track_id >> 16) as u8,
        (track_id >> 8) as u8, track_id as u8,

        0x00, 0x00, 0x00, 0x00,  // Reserved

        // Duration (movie timescale)
        (duration >> 24) as u8, (duration >> 16) as u8,
        (duration >> 8) as u8, duration as u8,

        // ... reserved, layer, alternate group, volume ...

        // Width (fixed point)
        (width_fixed >> 24) as u8, (width_fixed >> 16) as u8,
        (width_fixed >> 8) as u8, width_fixed as u8,

        // Height (fixed point)
        (height_fixed >> 24) as u8, (height_fixed >> 16) as u8,
        (height_fixed >> 8) as u8, height_fixed as u8,
    ]
}
```

**Fixed Point 16.16:**

```bash
Integer part: Upper 16 bits
Fractional part: Lower 16 bits

Example: 1280 × 720
  Width = 1280 << 16 = 0x05000000 = 83,886,080
  Height = 720 << 16 = 0x02D00000 = 47,185,920
```

### 2.6 mdhd (Media Header Box)

Defines media-specific timescale and duration.

**Important:** Movie timescale and Media timescale can differ!

**Structure:**

```bash
Size (4 bytes)
Type: 'mdhd'
Version (1 byte)
Flags (3 bytes)

Creation Time (4 bytes)
Modification Time (4 bytes)
Timescale (4 bytes) - This media's timescale
Duration (4 bytes) - Duration in this timescale units

Language (2 bytes) - ISO 639-2/T (packed)
Pre-defined (2 bytes)
```

**Language Encoding:**

```bash
Three 5-bit characters (ISO 639-2/T)
Example: "und" (undefined)
  'u' = 0x15, 'n' = 0x0E, 'd' = 0x04
  packed = 0x55C4
```

**Code Generation:**

```rust
// Video mdhd
let duration = samples.len() as u32 * 3000;  // 90kHz timescale
mdia.extend_from_slice(&[
    0x00, 0x00, 0x00, 0x20,  // Size: 32 bytes
    b'm', b'd', b'h', b'd',
    0x00,                     // Version
    0x00, 0x00, 0x00,        // Flags

    0x00, 0x00, 0x00, 0x00,  // Creation time
    0x00, 0x00, 0x00, 0x00,  // Modification time

    0x00, 0x01, 0x5F, 0x90,  // Timescale: 90000 Hz

    (duration >> 24) as u8,   // Duration
    (duration >> 16) as u8,
    (duration >> 8) as u8,
    duration as u8,

    0x55, 0xC4,              // Language: "und"
    0x00, 0x00,              // Pre-defined
]);

// Audio mdhd
let duration = samples.len() as u32 * 1920;  // 90kHz timescale
// AAC: 1024 samples @ 48kHz = 1920 in 90kHz
```

### 2.7 hdlr (Handler Reference Box)

Defines the media type.

**Structure:**

```bash
Size (4 bytes)
Type: 'hdlr'
Version (1 byte)
Flags (3 bytes)

Pre-defined (4 bytes)
Handler Type (4 bytes)
  'vide': Video track
  'soun': Audio track
  'hint': Hint track
Reserved (12 bytes)
Name (null-terminated string)
```

**Code Generation:**

```rust
// Video hdlr
mdia.extend_from_slice(&[
    0x00, 0x00, 0x00, 0x21,  // Size: 33 bytes
    b'h', b'd', b'l', b'r',
    0x00,                     // Version
    0x00, 0x00, 0x00,        // Flags

    0x00, 0x00, 0x00, 0x00,  // Pre-defined

    b'v', b'i', b'd', b'e',  // Handler type: video

    0x00, 0x00, 0x00, 0x00,  // Reserved
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,

    0x00, 0x00,              // Name (empty, null-terminated)
]);

// Audio hdlr
// Handler type: 'soun'
```

### 2.8 vmhd / smhd (Media Header)

**vmhd (Video Media Header):**

```bash
Size (4 bytes)
Type: 'vmhd'
Version (1 byte)
Flags (3 bytes) - 0x000001 (required)

Graphics Mode (2 bytes) - 0 (copy)
Opcolor (6 bytes) - RGB (0, 0, 0)
```

**smhd (Sound Media Header):**

```bash
Size (4 bytes)
Type: 'smhd'
Version (1 byte)
Flags (3 bytes)

Balance (2 bytes) - 0 (center)
Reserved (2 bytes)
```

### 2.9 stbl (Sample Table Box)

Core box containing all information about samples (frames).

**Contained Boxes:**

- stsd: Sample description (codec information)
- stts: Time-to-Sample (playback time)
- stsc: Sample-to-Chunk (chunk structure)
- stsz: Sample Size (individual sample sizes)
- stco: Chunk Offset (data location)
- ctts: Composition Time-to-Sample (display time, optional)

### 2.10 stsd (Sample Description Box)

Contains codec and format information.

**Structure:**

```bash
Size (4 bytes)
Type: 'stsd'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes) - Usually 1

[For each Entry]
  Sample Entry Box
    Video: avc1, hvc1, etc.
    Audio: mp4a, etc.
```

#### 2.10.1 avc1 (H.264 Sample Entry)

**Structure:**

```bash
Size (4 bytes)
Type: 'avc1'

Reserved (6 bytes) - 0
Data Reference Index (2 bytes) - 1

Pre-defined (2 bytes)
Reserved (2 bytes)
Pre-defined (12 bytes)

Width (2 bytes)
Height (2 bytes)

Horizontal Resolution (4 bytes) - 0x00480000 (72 dpi)
Vertical Resolution (4 bytes) - 0x00480000 (72 dpi)

Reserved (4 bytes)
Frame Count (2 bytes) - 1

Compressor Name (32 bytes) - Pascal string
  [1 byte length][name][padding]

Depth (2 bytes) - 0x0018 (24-bit color)
Pre-defined (2 bytes) - -1 (0xFFFF)

[Extension Boxes]
  avcC: AVC Decoder Configuration
  pasp: Pixel Aspect Ratio (optional)
  colr: Color Information (optional)
```

**Code Generation:**

```rust
fn build_video_stsd(media_data: &MediaData) -> io::Result<Vec<u8>> {
    let mut stsd = vec![
        0x00,                     // Version
        0x00, 0x00, 0x00,        // Flags
        0x00, 0x00, 0x00, 0x01,  // Entry count: 1
    ];

    // avc1 sample entry
    let mut avc1 = vec![
        // Reserved (6) + Data reference index (2)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,  // Data reference index

        // Pre-defined + Reserved
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,

        // Width & Height
        (media_data.width >> 8) as u8, media_data.width as u8,
        (media_data.height >> 8) as u8, media_data.height as u8,

        // Horizontal resolution: 72 dpi
        0x00, 0x48, 0x00, 0x00,
        // Vertical resolution: 72 dpi
        0x00, 0x48, 0x00, 0x00,

        // Reserved
        0x00, 0x00, 0x00, 0x00,

        // Frame count
        0x00, 0x01,

        // Compressor name (32 bytes)
        0x00, // Length: 0 (empty)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // ... (31 bytes total)

        // Depth: 24-bit
        0x00, 0x18,

        // Pre-defined: -1
        0xFF, 0xFF,
    ];

    // Add avcC box
    if let (Some(sps), Some(pps)) = (&media_data.sps, &media_data.pps) {
        avc1.extend_from_slice(&build_avcc(sps, pps));
    }

    // Complete avc1 box
    let avc1_size = 8 + avc1.len();
    let mut avc1_box = Vec::new();
    avc1_box.extend_from_slice(&(avc1_size as u32).to_be_bytes());
    avc1_box.extend_from_slice(b"avc1");
    avc1_box.extend_from_slice(&avc1);

    stsd.extend_from_slice(&avc1_box);

    // Complete stsd box
    let stsd_size = 8 + stsd.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(stsd_size as u32).to_be_bytes());
    result.extend_from_slice(b"stsd");
    result.extend_from_slice(&stsd);

    Ok(result)
}
```

#### 2.10.2 avcC (AVC Decoder Configuration)

Contains SPS/PPS required for H.264 decoder initialization.

**Structure:**

```bash
Size (4 bytes)
Type: 'avcC'

Configuration Version (1 byte) - 1
AVCProfileIndication (1 byte) - profile_idc from SPS
Profile Compatibility (1 byte) - constraint flags from SPS
AVCLevelIndication (1 byte) - level_idc from SPS

Length Size Minus One (6 bits reserved + 2 bits) - Usually 3 (4 bytes)

Num of SPS (5 bits reserved + 3 bits)
[For each SPS]
  SPS Length (2 bytes)
  SPS NAL Unit (variable)

Num of PPS (1 byte)
[For each PPS]
  PPS Length (2 bytes)
  PPS NAL Unit (variable)
```

**Code Generation:**

```rust
fn build_avcc(sps: &[u8], pps: &[u8]) -> Vec<u8> {
    let mut avcc = vec![
        0x01,        // Configuration version
        sps[1],      // AVCProfileIndication
        sps[2],      // Profile compatibility
        sps[3],      // AVCLevelIndication
        0xFF,        // 6 bits reserved (111111) + length_size_minus_one (11)
        0xE1,        // 3 bits reserved (111) + num_of_sps (00001)
    ];

    // SPS
    let sps_length = sps.len() as u16;
    avcc.push((sps_length >> 8) as u8);
    avcc.push(sps_length as u8);
    avcc.extend_from_slice(sps);

    // PPS count
    avcc.push(0x01);  // 1 PPS

    // PPS
    let pps_length = pps.len() as u16;
    avcc.push((pps_length >> 8) as u8);
    avcc.push(pps_length as u8);
    avcc.extend_from_slice(pps);

    // Complete avcC box
    let avcc_size = 8 + avcc.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(avcc_size as u32).to_be_bytes());
    result.extend_from_slice(b"avcC");
    result.extend_from_slice(&avcc);

    result
}
```

**Important:** MP4 uses **AVCC format** for H.264, not Annex B format.

- **Annex B**: Start code (0x00000001) + NAL
- **AVCC**: Length (4 bytes) + NAL (no start code)

**Conversion Code:**

```rust
fn convert_annexb_to_avcc(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Find start code
        if i + 3 < data.len()
            && data[i] == 0x00
            && data[i + 1] == 0x00
            && data[i + 2] == 0x00
            && data[i + 3] == 0x01
        {
            i += 4;  // 4-byte start code
        } else if i + 2 < data.len()
            && data[i] == 0x00
            && data[i + 1] == 0x00
            && data[i + 2] == 0x01
        {
            i += 3;  // 3-byte start code
        } else {
            i += 1;
            continue;
        }

        // Find NAL size
        let nal_start = i;
        let mut nal_end = nal_start;

        while nal_end + 2 < data.len() {
            if (data[nal_end] == 0x00
                && data[nal_end + 1] == 0x00
                && data[nal_end + 2] == 0x01)
                || (nal_end + 3 < data.len()
                    && data[nal_end] == 0x00
                    && data[nal_end + 1] == 0x00
                    && data[nal_end + 2] == 0x00
                    && data[nal_end + 3] == 0x01)
            {
                break;
            }
            nal_end += 1;
        }

        if nal_end > data.len() {
            nal_end = data.len();
        }

        let nal_size = nal_end - nal_start;

        // Write in AVCC format: [Length][NAL]
        result.extend_from_slice(&(nal_size as u32).to_be_bytes());
        result.extend_from_slice(&data[nal_start..nal_end]);

        i = nal_end;
    }

    result
}
```

#### 2.10.3 mp4a (AAC Sample Entry)

**Structure:**

```bash
Size (4 bytes)
Type: 'mp4a'

Reserved (6 bytes)
Data Reference Index (2 bytes) - 1

Version (2 bytes) - 0
Revision (2 bytes)
Vendor (4 bytes)

Channel Count (2 bytes) - 2 (stereo)
Sample Size (2 bytes) - 16 bits
Pre-defined (2 bytes)
Reserved (2 bytes)

Sample Rate (4 bytes) - Fixed point 16.16
  Example: 48000 Hz = 48000 << 16

[Extension Boxes]
  esds: Elementary Stream Descriptor (required)
```

**Code Generation:**

```rust
fn build_audio_stsd() -> io::Result<Vec<u8>> {
    let mut stsd = vec![
        0x00,                     // Version
        0x00, 0x00, 0x00,        // Flags
        0x00, 0x00, 0x00, 0x01,  // Entry count: 1
    ];

    let mut mp4a = vec![
        // Reserved (6) + Data reference index (2)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,

        // Version + Revision + Vendor
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,

        // Channel count: 2 (stereo)
        0x00, 0x02,

        // Sample size: 16 bits
        0x00, 0x10,

        // Pre-defined + Reserved
        0x00, 0x00, 0x00, 0x00,

        // Sample rate: 48000 Hz (16.16 fixed point)
        0xBB, 0x80, 0x00, 0x00,  // 48000 << 16
    ];

    // Add esds box
    mp4a.extend_from_slice(&build_esds());

    // ...
}
```

#### 2.10.4 esds (Elementary Stream Descriptor)

Contains AAC decoder configuration.

**Structure (MP4 descriptor format):**

```bash
Size (4 bytes)
Type: 'esds'
Version (1 byte)
Flags (3 bytes)

ES_Descriptor (tag 0x03)
  Tag (1 byte) - 0x03
  Length (variable)
  ES_ID (2 bytes)
  Flags (1 byte)

  DecoderConfigDescriptor (tag 0x04)
    Tag (1 byte) - 0x04
    Length (variable)
    Object Type (1 byte) - 0x40 (Audio ISO/IEC 14496-3)
    Stream Type (1 byte) - 0x15 (audio stream)
    Buffer Size DB (3 bytes)
    Max Bitrate (4 bytes)
    Avg Bitrate (4 bytes)

    DecoderSpecificInfo (tag 0x05)
      Tag (1 byte) - 0x05
      Length (variable)
      Audio Specific Config (variable)
        [AAC configuration bits]

  SLConfigDescriptor (tag 0x06)
    Tag (1 byte) - 0x06
    Length (variable)
    Pre-defined (1 byte) - 0x02
```

**Audio Specific Config (AAC-LC, 48kHz, Stereo):**

```bash
5 bits: Audio Object Type - 2 (AAC-LC)
4 bits: Sampling Frequency Index - 3 (48000 Hz)
4 bits: Channel Configuration - 2 (Stereo)
3 bits: Frame Length Flag + depends on core coder + extension flag

Example: AAC-LC, 48kHz, Stereo
  Binary: 00010 0011 0010 000
  Hex: 0x11 0x90
```

**Code Generation:**

```rust
fn build_esds() -> Vec<u8> {
    let mut esds = vec![
        0x00,                     // Version
        0x00, 0x00, 0x00,        // Flags

        // ES_Descriptor (tag 0x03)
        0x03,                     // Tag
        0x80, 0x80, 0x80, 0x22,  // Length: 34 bytes (variable length encoding)

        0x00, 0x00,              // ES_ID: 0
        0x00,                     // Flags

        // DecoderConfigDescriptor (tag 0x04)
        0x04,                     // Tag
        0x80, 0x80, 0x80, 0x14,  // Length: 20 bytes

        0x40,                     // Object type: Audio ISO/IEC 14496-3
        0x15,                     // Stream type: Audio

        0x00, 0x00, 0x00,        // Buffer size DB: 0

        0x00, 0x00, 0x00, 0x00,  // Max bitrate: 0 (variable)
        0x00, 0x00, 0x00, 0x00,  // Avg bitrate: 0 (variable)

        // DecoderSpecificInfo (tag 0x05)
        0x05,                     // Tag
        0x80, 0x80, 0x80, 0x02,  // Length: 2 bytes

        // Audio Specific Config: AAC-LC, 48kHz, Stereo
        0x11, 0x90,

        // SLConfigDescriptor (tag 0x06)
        0x06,                     // Tag
        0x80, 0x80, 0x80, 0x01,  // Length: 1 byte
        0x02,                     // Pre-defined: MP4
    ];

    let esds_size = 8 + esds.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(esds_size as u32).to_be_bytes());
    result.extend_from_slice(b"esds");
    result.extend_from_slice(&esds);

    result
}
```

### 2.11 stts (Time-to-Sample Box)

Defines the playback duration of each sample.

**Structure:**

```bash
Size (4 bytes)
Type: 'stts'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes)

[For each Entry]
  Sample Count (4 bytes) - Number of samples with this duration
  Sample Delta (4 bytes) - Duration of each sample (in timescale units)
```

**Examples:**

```bash
Video (30fps, 90kHz timescale):
  Sample Count: 700
  Sample Delta: 3000  (90000 / 30)

Audio (AAC, 48kHz, 90kHz timescale):
  Sample Count: 1095
  Sample Delta: 1920  (1024 samples @ 48kHz in 90kHz)
                      = (1024 / 48000) * 90000
```

**Code Generation:**

```rust
// Video stts
let sample_count = samples.len() as u32;
let mut stts = vec![
    0x00,                     // Version
    0x00, 0x00, 0x00,        // Flags

    0x00, 0x00, 0x00, 0x01,  // Entry count: 1

    // Entry 1
    (sample_count >> 24) as u8,  // Sample count
    (sample_count >> 16) as u8,
    (sample_count >> 8) as u8,
    sample_count as u8,

    0x00, 0x00, 0x0B, 0xB8,  // Sample delta: 3000
];

// Audio stts
let sample_count = samples.len() as u32;
let mut stts = vec![
    // ... version, flags, entry count ...

    // Entry 1
    (sample_count >> 24) as u8,
    (sample_count >> 16) as u8,
    (sample_count >> 8) as u8,
    sample_count as u8,

    0x00, 0x00, 0x07, 0x80,  // Sample delta: 1920
];
```

### 2.12 stsc (Sample-to-Chunk Box)

Defines how samples are arranged in chunks.

**Structure:**

```bash
Size (4 bytes)
Type: 'stsc'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes)

[For each Entry]
  First Chunk (4 bytes) - Chunk number where this setting starts (from 1)
  Samples Per Chunk (4 bytes) - Number of samples per chunk
  Sample Description Index (4 bytes) - stsd entry index (from 1)
```

**Single Chunk Approach (This Project):**

All samples in one chunk for improved compatibility.

```bash
Entry Count: 1
  First Chunk: 1
  Samples Per Chunk: 700 (all samples)
  Sample Description Index: 1
```

**Code Generation:**

```rust
// Video stsc - all samples in 1 chunk
stbl.extend_from_slice(&[
    0x00, 0x00, 0x00, 0x1C,  // Size: 28 bytes
    b's', b't', b's', b'c',
    0x00,                     // Version
    0x00, 0x00, 0x00,        // Flags

    0x00, 0x00, 0x00, 0x01,  // Entry count: 1

    // Entry 1
    0x00, 0x00, 0x00, 0x01,  // First chunk: 1

    (sample_count >> 24) as u8,  // Samples per chunk: all
    (sample_count >> 16) as u8,
    (sample_count >> 8) as u8,
    sample_count as u8,

    0x00, 0x00, 0x00, 0x01,  // Sample description index: 1
]);
```

### 2.13 stsz (Sample Size Box)

Defines the size of each sample in bytes.

**Structure:**

```bash
Size (4 bytes)
Type: 'stsz'
Version (1 byte)
Flags (3 bytes)

Sample Size (4 bytes) - 0 means variable size
Sample Count (4 bytes)

[Only when Sample Size is 0]
  [For each sample]
    Entry Size (4 bytes)
```

**Video/Audio usually have variable sizes:**

```bash
Sample Size: 0 (variable)
Sample Count: 700

Entry 1 Size: 45023
Entry 2 Size: 12456
Entry 3 Size: 8912
...
```

**Code Generation:**

```rust
let mut stsz = vec![
    0x00,                     // Version
    0x00, 0x00, 0x00,        // Flags

    0x00, 0x00, 0x00, 0x00,  // Sample size: 0 (variable)

    (sample_count >> 24) as u8,  // Sample count
    (sample_count >> 16) as u8,
    (sample_count >> 8) as u8,
    sample_count as u8,
];

// Add each sample size
for sample in samples {
    let size = sample.len() as u32;
    stsz.push((size >> 24) as u8);
    stsz.push((size >> 16) as u8);
    stsz.push((size >> 8) as u8);
    stsz.push(size as u8);
}
```

### 2.14 stco (Chunk Offset Box)

Defines the location of each chunk within the file.

**Structure:**

```bash
Size (4 bytes)
Type: 'stco'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes)

[For each Entry]
  Chunk Offset (4 bytes) - Byte offset from file start
```

**Important:** Offsets are **absolute positions**!

```bash
File structure:
[0-27] ftyp (28 bytes)
[28-X] moov (variable)
[X+1-X+8] mdat header (8 bytes)
[X+9-...] mdat data

Video chunk offset = ftyp_size + moov_size + 8
Audio chunk offset = video chunk offset + video data size
```

**Code Generation:**

```rust
// Video stco
let base_offset = ftyp_size + moov_size + mdat_header_size;
let chunk_count = 1u32;  // Single chunk

let mut stco = vec![
    0x00,                     // Version
    0x00, 0x00, 0x00,        // Flags

    0x00, 0x00, 0x00, 0x01,  // Chunk count: 1
];

// Chunk offset
stco.extend_from_slice(&(base_offset as u32).to_be_bytes());

// Audio stco
let audio_offset = base_offset + video_data_end;
stco.extend_from_slice(&(audio_offset as u32).to_be_bytes());
```

### 2.15 ctts (Composition Time-to-Sample Box)

Defines display time offsets (when B-frames exist).

**Structure:**

```bash
Size (4 bytes)
Type: 'ctts'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes)

[For each Entry]
  Sample Count (4 bytes)
  Sample Offset (4 bytes) - Composition offset (signed in version 1)
```

**Composition Time = Decode Time + Offset**

```bash
Example: With B-frames
  Frame 0 (I): DTS=0,   PTS=2000, Offset=2000
  Frame 1 (P): DTS=1000, PTS=4000, Offset=3000
  Frame 2 (B): DTS=2000, PTS=1000, Offset=-1000
  Frame 3 (B): DTS=3000, PTS=3000, Offset=0
  Frame 4 (P): DTS=4000, PTS=6000, Offset=2000
```

**Code Generation:**

```rust
fn calculate_composition_offsets(
    frame_timestamps: &[(Option<u64>, Option<u64>)],
    global_min_pts: u64,
) -> Vec<i32> {
    frame_timestamps
        .iter()
        .map(|(pts, dts)| {
            match (pts, dts) {
                (Some(p), Some(d)) => {
                    let adjusted_pts = p.saturating_sub(global_min_pts);
                    let adjusted_dts = d.saturating_sub(global_min_pts);
                    (adjusted_pts as i64 - adjusted_dts as i64) as i32
                }
                (Some(p), None) => 0,
                _ => 0,
            }
        })
        .collect()
}

// Generate ctts box (only when offsets exist)
if !composition_offsets.is_empty() && composition_offsets.iter().any(|&o| o != 0) {
    let mut ctts = vec![
        0x00,                     // Version
        0x00, 0x00, 0x00,        // Flags

        (sample_count >> 24) as u8,  // Entry count
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8,
    ];

    for offset in composition_offsets {
        ctts.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x01,  // Sample count: 1
        ]);
        ctts.extend_from_slice(&(*offset as u32).to_be_bytes());
    }

    // ...
}
```

### 2.16 edts / elst (Edit List)

Adjusts media playback sections and timing.

**Structure:**

```bash
edts (Edit Box)
└─ elst (Edit List Box)
   Size (4 bytes)
   Type: 'elst'
   Version (1 byte)
   Flags (3 bytes)

   Entry Count (4 bytes)

   [For each Entry]
     Segment Duration (4/8 bytes) - Movie timescale
     Media Time (4/8 bytes) - Media timescale, -1=empty
     Media Rate (4 bytes) - Fixed point 16.16
```

**Use Cases:**

- Audio/video start time synchronization
- Insert empty segments
- Adjust playback speed

**Code Generation (when audio starts late):**

```rust
if let Some(Some(first_audio_pts)) = media_data.audio_timestamps.first() {
    if *first_audio_pts > global_min_pts {
        let delay = first_audio_pts - global_min_pts;

        // Add Edit List
        let mut edts = Vec::new();

        // elst
        let mut elst = vec![
            0x00,                     // Version
            0x00, 0x00, 0x00,        // Flags
            0x00, 0x00, 0x00, 0x02,  // Entry count: 2
        ];

        // Entry 1: Empty edit (delay)
        elst.extend_from_slice(&(delay as u32).to_be_bytes());
        elst.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);  // Media time: -1
        elst.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]);  // Rate: 1.0

        // Entry 2: Normal playback
        elst.extend_from_slice(&(duration as u32).to_be_bytes());
        elst.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);  // Media time: 0
        elst.extend_from_slice(&[0x00, 0x01, 0x00, 0x00]);  // Rate: 1.0

        // ...
    }
}
```

### 2.17 mdat (Media Data Box)

Contains actual media data.

**Structure:**

```bash
Size (4 bytes)
Type: 'mdat'
Data (Size - 8 bytes)
  [Video frame 1]
  [Video frame 2]
  ...
  [Audio frame 1]
  [Audio frame 2]
  ...
```

**Characteristics:**

- Largest box (usually most of the file)
- Contains only pure data
- Referenced by stco

**Code Generation:**

```rust
// 1. Prepare data
let mut mdat_data = Vec::new();

// Video data
for sample in &video_samples {
    mdat_data.extend_from_slice(sample);
}
let video_data_end = mdat_data.len();

// Audio data
for sample in audio_samples {
    mdat_data.extend_from_slice(sample);
}

// 2. Create mdat box
let mdat_size = 8 + mdat_data.len();
mp4_buffer.extend_from_slice(&(mdat_size as u32).to_be_bytes());
mp4_buffer.extend_from_slice(b"mdat");
mp4_buffer.extend_from_slice(&mdat_data);
```

## 3. MP4 Generation Flow

### 3.1 Overall Process

```bash
1. Prepare video/audio data
   ├─ Annex B → AVCC conversion (video)
   └─ Remove ADTS (audio)
   ↓
2. Assemble mdat data
   ├─ Video frames
   └─ Audio frames
   ↓
3. Calculate offsets
   ├─ ftyp size
   ├─ moov size (temporary calculation)
   ├─ mdat header
   └─ Each chunk offset
   ↓
4. Generate moov box
   ├─ mvhd
   ├─ trak (video)
   │   └─ stbl (stsd, stts, stsc, stsz, stco, ctts)
   └─ trak (audio)
       └─ stbl
   ↓
5. Assemble final file
   [ftyp][moov][mdat]
```

### 3.2 Main Function in Code

```rust
pub fn create_mp4(media_data: MediaData) -> io::Result<Vec<u8>> {
    // 1. Prepare video data
    let frames = split_into_frames(&media_data.video_stream);
    let mut video_samples = Vec::new();

    for frame in frames.iter() {
        let avcc_frame = convert_annexb_to_avcc(frame);
        video_samples.push(avcc_frame);
    }

    // 2. Audio data
    let audio_samples = &media_data.audio_frames;

    // 3. Assemble mdat data
    let mut mdat_data = Vec::new();
    for sample in &video_samples {
        mdat_data.extend_from_slice(sample);
    }
    let video_data_end = mdat_data.len();

    for sample in audio_samples {
        mdat_data.extend_from_slice(sample);
    }

    // 4. Calculate offsets
    let ftyp_size = 28;
    let mdat_header_size = 8;

    // Temporary build to calculate moov size
    let moov_box = build_moov(
        &media_data,
        &video_samples,
        audio_samples,
        ftyp_size,
        0,  // temporary
        mdat_header_size,
        video_data_end,
    )?;

    let moov_size = moov_box.len();

    // 5. Regenerate moov with accurate offsets
    let moov_box = build_moov(
        &media_data,
        &video_samples,
        audio_samples,
        ftyp_size,
        moov_size,
        mdat_header_size,
        video_data_end,
    )?;

    // 6. Assemble final file
    let mut mp4_buffer = Vec::new();

    // ftyp
    mp4_buffer.extend_from_slice(&[ /* ... */ ]);

    // moov
    mp4_buffer.extend_from_slice(&moov_box);

    // mdat
    let mdat_size = 8 + mdat_data.len();
    mp4_buffer.extend_from_slice(&(mdat_size as u32).to_be_bytes());
    mp4_buffer.extend_from_slice(b"mdat");
    mp4_buffer.extend_from_slice(&mdat_data);

    Ok(mp4_buffer)
}
```

## 4. Timescale and Duration

### 4.1 Timescale Concept

**Timescale**: Number of units representing 1 second

```bash
90kHz timescale = 1 second = 90000 units
48kHz timescale = 1 second = 48000 units
```

### 4.2 This Project's Timescale Policy

**Unified 90kHz for all tracks:**

```bash
Movie timescale (mvhd): 90000 Hz
Video media timescale (mdhd): 90000 Hz
Audio media timescale (mdhd): 90000 Hz  ← Note: Not 48000!
```

**Reason:** Compatibility and simplified synchronization

### 4.3 Duration Calculation

**Video (30fps):**

```bash
1 frame = 1/30 second = 90000/30 = 3000 units

700 frames = 700 × 3000 = 2,100,000 units
          = 2,100,000 / 90000 = 23.33 seconds
```

**Audio (AAC, 48kHz):**

```bash
1 frame = 1024 samples @ 48kHz
        = 1024/48000 seconds
        = 0.021333 seconds
        = 0.021333 × 90000 = 1920 units

1095 frames = 1095 × 1920 = 2,102,400 units
           = 2,102,400 / 90000 = 23.36 seconds
```

### 4.4 Timestamp Normalization

**Global Minimum PTS:**

```rust
let video_min_pts = media_data
    .frame_timestamps
    .iter()
    .filter_map(|(pts, _)| *pts)
    .min()
    .unwrap_or(0);

let audio_min_pts = media_data
    .audio_timestamps
    .iter()
    .filter_map(|&pts| pts)
    .min()
    .unwrap_or(0);

let global_min_pts = video_min_pts.min(audio_min_pts);

// Normalize by subtracting global_min_pts from all timestamps to start from 0
```

## 5. Summary

### MP4 Hierarchical Structure

```bash
MP4 File
├─ ftyp - File type
├─ moov - Metadata container
│  ├─ mvhd - Overall video information
│  └─ trak - Each track
│     ├─ tkhd - Track information
│     └─ mdia
│        ├─ mdhd - Media timescale/duration
│        ├─ hdlr - Media type
│        └─ minf
│           ├─ vmhd/smhd - Video/audio header
│           └─ stbl - Sample table
│              ├─ stsd - Codec information (avc1/mp4a)
│              ├─ stts - Playback time
│              ├─ stsc - Chunk structure
│              ├─ stsz - Sample sizes
│              ├─ stco - Data locations
│              └─ ctts - Display time offset
└─ mdat - Actual media data
```

### Key Concepts

1. **Box Structure**: Recursive structure of Size, Type, and Data
2. **Timescale**: Time representation unit (unified at 90kHz)
3. **Sample**: Individual frame (video) or audio block
4. **Chunk**: Group of consecutive samples
5. **Offset**: Absolute position within file
6. **Duration**: Playback time in timescale units
7. **PTS/DTS**: Presentation/Decoding timestamps
8. **Composition Offset**: PTS - DTS
