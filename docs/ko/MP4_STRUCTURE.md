# MP4 파일 구조

MP4 (MPEG-4 Part 14)는 ISO Base Media File Format을 기반으로 하는 컨테이너 형식입니다. 비디오, 오디오, 자막 등을 저장할 수 있습니다.

## 1. 기본 개념

### 1.1 Box (Atom) 구조

MP4는 "Box"라는 기본 단위로 구성됩니다. 각 Box는 다음 구조를 가집니다:

```bash
[4 bytes] Size - Box 전체 크기 (헤더 포함)
[4 bytes] Type - Box 타입 (FourCC)
[Size-8 bytes] Data - Box 데이터 (중첩된 Box 포함 가능)
```

**특수 케이스:**

- Size = 0: 파일 끝까지
- Size = 1: 다음 8바이트가 64-bit 크기

**코드에서 Box 생성:**

```rust
// 예: stts box 생성
let stts_size = 8 + stts.len();
let mut stts_box = Vec::new();
stts_box.extend_from_slice(&(stts_size as u32).to_be_bytes());   // Size
stts_box.extend_from_slice(b"stts");                             // Type
stts_box.extend_from_slice(&stts);                               // Data
```

### 1.2 전체 파일 구조

```bash
MP4 File
├─ ftyp (File Type Box)
├─ moov (Movie Box) - 메타데이터
│   ├─ mvhd (Movie Header)
│   ├─ trak (Track) - 비디오
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
│   └─ trak (Track) - 오디오
│       └─ ... (비디오와 유사)
└─ mdat (Media Data Box) - 실제 데이터
    ├─ [비디오 프레임들...]
    └─ [오디오 프레임들...]
```

## 2. 주요 Box 상세 설명

### 2.1 ftyp (File Type Box)

파일의 브랜드와 호환성 정보를 담습니다.

**구조:**

```bash
Size (4 bytes)
Type: 'ftyp'
Major Brand (4 bytes) - 주 브랜드
Minor Version (4 bytes) - 버전
Compatible Brands (4 bytes × N) - 호환 브랜드 목록
```

**코드에서 생성:**

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

**주요 브랜드:**

- `isom`: ISO Base Media File Format
- `iso2`: ISO Base Media File Format version 2
- `mp41`: MPEG-4 version 1
- `avc1`: H.264/AVC

### 2.2 moov (Movie Box)

모든 메타데이터를 담는 컨테이너 Box입니다.

**특징:**

- 파일의 처음이나 끝에 위치 가능
- 처음에 있으면 빠른 재생 시작 (fast start)
- 모든 트랙의 정보 포함

### 2.3 mvhd (Movie Header Box)

전체 영상의 속성을 정의합니다.

**구조 (Version 0, 108 bytes):**

```bash
Size (4 bytes)
Type: 'mvhd'
Version (1 byte) - 0
Flags (3 bytes) - 0

Creation Time (4 bytes) - 1904년 1월 1일부터의 초
Modification Time (4 bytes)
Timescale (4 bytes) - 1초를 나타내는 단위 (예: 90000)
Duration (4 bytes) - Timescale 단위의 재생 시간

Preferred Rate (4 bytes) - 재생 속도 (1.0 = 0x00010000)
Preferred Volume (2 bytes) - 볼륨 (1.0 = 0x0100)
Reserved (10 bytes)
Matrix (36 bytes) - 비디오 변환 매트릭스
Pre-defined (24 bytes)
Next Track ID (4 bytes) - 다음 트랙 ID
```

**코드에서 생성:**

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
        // Duration (4 bytes) - 계산됨
        duration.to_be_bytes()[0], duration.to_be_bytes()[1],
        duration.to_be_bytes()[2], duration.to_be_bytes()[3],

        0x00, 0x01, 0x00, 0x00,  // Preferred rate: 1.0
        0x01, 0x00,              // Preferred volume: 1.0

        // Reserved (10 bytes)
        0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,

        // Matrix (36 bytes) - Identity matrix
        0x00, 0x01, 0x00, 0x00,  // [1.0, 0, 0]
        // ... (나머지 매트릭스 값들)

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

**Duration 계산:**

```bash
비디오: frame_count × 3000  (30fps, 90kHz timescale)
       = frame_count × (90000 / 30)

예: 700 프레임 → 700 × 3000 = 2,100,000
   → 2,100,000 / 90000 = 23.33초
```

### 2.4 trak (Track Box)

각 미디어 트랙(비디오, 오디오 등)을 나타냅니다.

**하나의 MP4에 여러 trak 존재 가능:**

- Track 1: 비디오
- Track 2: 오디오
- Track 3: 자막 등

### 2.5 tkhd (Track Header Box)

개별 트랙의 속성을 정의합니다.

**구조 (Version 0):**

```bash
Size (4 bytes)
Type: 'tkhd'
Version (1 byte) - 0
Flags (3 bytes) - 0x000007 (enabled, in movie, in preview)

Creation Time (4 bytes)
Modification Time (4 bytes)
Track ID (4 bytes) - 트랙 식별자 (1부터 시작)
Reserved (4 bytes)
Duration (4 bytes) - Movie timescale 단위

Reserved (8 bytes)
Layer (2 bytes) - 비디오 레이어 (0)
Alternate Group (2 bytes) - 대체 그룹
Volume (2 bytes) - 오디오 볼륨 (비디오는 0)
Reserved (2 bytes)
Matrix (36 bytes) - 변환 매트릭스
Width (4 bytes) - Fixed point 16.16
Height (4 bytes) - Fixed point 16.16
```

**코드에서 생성:**

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
정수 부분: 상위 16 bits
소수 부분: 하위 16 bits

예: 1280 × 720
  Width = 1280 << 16 = 0x05000000 = 83,886,080
  Height = 720 << 16 = 0x02D00000 = 47,185,920
```

### 2.6 mdhd (Media Header Box)

미디어별 timescale과 duration을 정의합니다.

**중요:** Movie timescale과 Media timescale이 다를 수 있습니다!

**구조:**

```bash
Size (4 bytes)
Type: 'mdhd'
Version (1 byte)
Flags (3 bytes)

Creation Time (4 bytes)
Modification Time (4 bytes)
Timescale (4 bytes) - 이 미디어의 timescale
Duration (4 bytes) - 이 timescale 단위의 duration

Language (2 bytes) - ISO 639-2/T (packed)
Pre-defined (2 bytes)
```

**Language 인코딩:**

```bash
3개의 5-bit 문자 (ISO 639-2/T)
예: "und" (undefined)
  'u' = 0x15, 'n' = 0x0E, 'd' = 0x04
  packed = 0x55C4
```

**코드에서 생성:**

```rust
// 비디오 mdhd
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

// 오디오 mdhd
let duration = samples.len() as u32 * 1920;  // 90kHz timescale
// AAC: 1024 samples @ 48kHz = 1920 in 90kHz
```

### 2.7 hdlr (Handler Reference Box)

미디어 타입을 정의합니다.

**구조:**

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

**코드에서 생성:**

```rust
// 비디오 hdlr
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

// 오디오 hdlr
// Handler type: 'soun'
```

### 2.8 vmhd / smhd (Media Header)

**vmhd (Video Media Header):**

```bash
Size (4 bytes)
Type: 'vmhd'
Version (1 byte)
Flags (3 bytes) - 0x000001 (필수)

Graphics Mode (2 bytes) - 0 (copy)
Opcolor (6 bytes) - RGB (0, 0, 0)
```

**smhd (Sound Media Header):**

```bash
Size (4 bytes)
Type: 'smhd'
Version (1 byte)
Flags (3 bytes)

Balance (2 bytes) - 0 (중앙)
Reserved (2 bytes)
```

### 2.9 stbl (Sample Table Box)

샘플(프레임)에 대한 모든 정보를 담는 핵심 Box입니다.

**포함 Box들:**

- stsd: 샘플 설명 (코덱 정보)
- stts: Time-to-Sample (재생 시간)
- stsc: Sample-to-Chunk (청크 구조)
- stsz: Sample Size (각 샘플 크기)
- stco: Chunk Offset (데이터 위치)
- ctts: Composition Time-to-Sample (표시 시간, 선택적)

### 2.10 stsd (Sample Description Box)

코덱과 포맷 정보를 담습니다.

**구조:**

```bash
Size (4 bytes)
Type: 'stsd'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes) - 보통 1

[각 Entry마다]
  Sample Entry Box
    비디오: avc1, hvc1 등
    오디오: mp4a 등
```

#### 2.10.1 avc1 (H.264 Sample Entry)

**구조:**

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
  pasp: Pixel Aspect Ratio (선택적)
  colr: Color Information (선택적)
```

**코드에서 생성:**

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

    // avcC box 추가
    if let (Some(sps), Some(pps)) = (&media_data.sps, &media_data.pps) {
        avc1.extend_from_slice(&build_avcc(sps, pps));
    }

    // avc1 box 완성
    let avc1_size = 8 + avc1.len();
    let mut avc1_box = Vec::new();
    avc1_box.extend_from_slice(&(avc1_size as u32).to_be_bytes());
    avc1_box.extend_from_slice(b"avc1");
    avc1_box.extend_from_slice(&avc1);

    stsd.extend_from_slice(&avc1_box);

    // stsd box 완성
    let stsd_size = 8 + stsd.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(stsd_size as u32).to_be_bytes());
    result.extend_from_slice(b"stsd");
    result.extend_from_slice(&stsd);

    Ok(result)
}
```

#### 2.10.2 avcC (AVC Decoder Configuration)

H.264 디코더 초기화에 필요한 SPS/PPS를 담습니다.

**구조:**

```bash
Size (4 bytes)
Type: 'avcC'

Configuration Version (1 byte) - 1
AVCProfileIndication (1 byte) - SPS의 profile_idc
Profile Compatibility (1 byte) - SPS의 constraint flags
AVCLevelIndication (1 byte) - SPS의 level_idc

Length Size Minus One (6 bits reserved + 2 bits) - 보통 3 (4 bytes)

Num of SPS (5 bits reserved + 3 bits)
[각 SPS마다]
  SPS Length (2 bytes)
  SPS NAL Unit (variable)

Num of PPS (1 byte)
[각 PPS마다]
  PPS Length (2 bytes)
  PPS NAL Unit (variable)
```

**코드에서 생성:**

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

    // avcC box 완성
    let avcc_size = 8 + avcc.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(avcc_size as u32).to_be_bytes());
    result.extend_from_slice(b"avcC");
    result.extend_from_slice(&avcc);

    result
}
```

**중요:** MP4에서 H.264는 Annex B 형식이 아닌 **AVCC 형식**을 사용합니다.

- **Annex B**: Start code (0x00000001) + NAL
- **AVCC**: Length (4 bytes) + NAL (start code 없음)

**변환 코드:**

```rust
fn convert_annexb_to_avcc(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Start code 찾기
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

        // NAL 크기 찾기
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

        // AVCC 형식으로 쓰기: [Length][NAL]
        result.extend_from_slice(&(nal_size as u32).to_be_bytes());
        result.extend_from_slice(&data[nal_start..nal_end]);

        i = nal_end;
    }

    result
}
```

#### 2.10.3 mp4a (AAC Sample Entry)

**구조:**

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
  예: 48000 Hz = 48000 << 16

[Extension Boxes]
  esds: Elementary Stream Descriptor (필수)
```

**코드에서 생성:**

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

    // esds box 추가
    mp4a.extend_from_slice(&build_esds());

    // ...
}
```

#### 2.10.4 esds (Elementary Stream Descriptor)

AAC 디코더 설정을 담습니다.

**구조 (MP4 descriptor 형식):**

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
        [AAC 설정 비트들]

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

예: AAC-LC, 48kHz, Stereo
  Binary: 00010 0011 0010 000
  Hex: 0x11 0x90
```

**코드에서 생성:**

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

각 샘플의 재생 시간(duration)을 정의합니다.

**구조:**

```bash
Size (4 bytes)
Type: 'stts'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes)

[각 Entry마다]
  Sample Count (4 bytes) - 이 duration을 가진 샘플 개수
  Sample Delta (4 bytes) - 각 샘플의 duration (timescale 단위)
```

**예시:**

```bash
비디오 (30fps, 90kHz timescale):
  Sample Count: 700
  Sample Delta: 3000  (90000 / 30)

오디오 (AAC, 48kHz, 90kHz timescale):
  Sample Count: 1095
  Sample Delta: 1920  (1024 samples @ 48kHz in 90kHz)
                      = (1024 / 48000) * 90000
```

**코드에서 생성:**

```rust
// 비디오 stts
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

// 오디오 stts
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

샘플들이 청크에 어떻게 배치되는지 정의합니다.

**구조:**

```bash
Size (4 bytes)
Type: 'stsc'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes)

[각 Entry마다]
  First Chunk (4 bytes) - 이 설정이 시작되는 청크 번호 (1부터)
  Samples Per Chunk (4 bytes) - 청크당 샘플 개수
  Sample Description Index (4 bytes) - stsd의 entry index (1부터)
```

**단일 청크 방식 (본 프로젝트):**

모든 샘플을 하나의 청크에 넣어 호환성을 높임.

```bash
Entry Count: 1
  First Chunk: 1
  Samples Per Chunk: 700 (모든 샘플)
  Sample Description Index: 1
```

**코드에서 생성:**

```rust
// 비디오 stsc - 모든 샘플을 1개 청크에
stbl.extend_from_slice(&[
    0x00, 0x00, 0x00, 0x1C,  // Size: 28 bytes
    b's', b't', b's', b'c',
    0x00,                     // Version
    0x00, 0x00, 0x00,        // Flags

    0x00, 0x00, 0x00, 0x01,  // Entry count: 1

    // Entry 1
    0x00, 0x00, 0x00, 0x01,  // First chunk: 1

    (sample_count >> 24) as u8,  // Samples per chunk: 전체
    (sample_count >> 16) as u8,
    (sample_count >> 8) as u8,
    sample_count as u8,

    0x00, 0x00, 0x00, 0x01,  // Sample description index: 1
]);
```

### 2.13 stsz (Sample Size Box)

각 샘플의 크기를 바이트 단위로 정의합니다.

**구조:**

```bash
Size (4 bytes)
Type: 'stsz'
Version (1 byte)
Flags (3 bytes)

Sample Size (4 bytes) - 0이면 가변 크기
Sample Count (4 bytes)

[Sample Size가 0일 때만]
  [각 샘플마다]
    Entry Size (4 bytes)
```

**비디오/오디오는 보통 가변 크기:**

```bash
Sample Size: 0 (가변)
Sample Count: 700

Entry 1 Size: 45023
Entry 2 Size: 12456
Entry 3 Size: 8912
...
```

**코드에서 생성:**

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

// 각 샘플 크기 추가
for sample in samples {
    let size = sample.len() as u32;
    stsz.push((size >> 24) as u8);
    stsz.push((size >> 16) as u8);
    stsz.push((size >> 8) as u8);
    stsz.push(size as u8);
}
```

### 2.14 stco (Chunk Offset Box)

각 청크의 파일 내 위치를 정의합니다.

**구조:**

```bash
Size (4 bytes)
Type: 'stco'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes)

[각 Entry마다]
  Chunk Offset (4 bytes) - 파일 시작부터의 바이트 오프셋
```

**중요:** 오프셋은 **절대 위치**입니다!

```bash
파일 구조:
[0-27] ftyp (28 bytes)
[28-X] moov (variable)
[X+1-X+8] mdat header (8 bytes)
[X+9-...] mdat data

비디오 청크 오프셋 = ftyp_size + moov_size + 8
오디오 청크 오프셋 = 비디오 청크 오프셋 + 비디오 데이터 크기
```

**코드에서 생성:**

```rust
// 비디오 stco
let base_offset = ftyp_size + moov_size + mdat_header_size;
let chunk_count = 1u32;  // 단일 청크

let mut stco = vec![
    0x00,                     // Version
    0x00, 0x00, 0x00,        // Flags

    0x00, 0x00, 0x00, 0x01,  // Chunk count: 1
];

// 청크 오프셋
stco.extend_from_slice(&(base_offset as u32).to_be_bytes());

// 오디오 stco
let audio_offset = base_offset + video_data_end;
stco.extend_from_slice(&(audio_offset as u32).to_be_bytes());
```

### 2.15 ctts (Composition Time-to-Sample Box)

표시 시간 오프셋을 정의합니다 (B-프레임이 있을 때).

**구조:**

```bash
Size (4 bytes)
Type: 'ctts'
Version (1 byte)
Flags (3 bytes)

Entry Count (4 bytes)

[각 Entry마다]
  Sample Count (4 bytes)
  Sample Offset (4 bytes) - Composition offset (signed in version 1)
```

**Composition Time = Decode Time + Offset**

```bash
예: B-프레임이 있는 경우
  Frame 0 (I): DTS=0,   PTS=2000, Offset=2000
  Frame 1 (P): DTS=1000, PTS=4000, Offset=3000
  Frame 2 (B): DTS=2000, PTS=1000, Offset=-1000
  Frame 3 (B): DTS=3000, PTS=3000, Offset=0
  Frame 4 (P): DTS=4000, PTS=6000, Offset=2000
```

**코드에서 생성:**

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

// ctts box 생성 (오프셋이 있을 때만)
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

미디어의 재생 구간과 타이밍을 조정합니다.

**구조:**

```bash
edts (Edit Box)
└─ elst (Edit List Box)
   Size (4 bytes)
   Type: 'elst'
   Version (1 byte)
   Flags (3 bytes)

   Entry Count (4 bytes)

   [각 Entry마다]
     Segment Duration (4/8 bytes) - Movie timescale
     Media Time (4/8 bytes) - Media timescale, -1=empty
     Media Rate (4 bytes) - Fixed point 16.16
```

**용도:**

- 오디오/비디오 시작 시간 동기화
- 빈 구간 삽입
- 재생 속도 조정

**코드에서 생성 (오디오가 늦게 시작할 때):**

```rust
if let Some(Some(first_audio_pts)) = media_data.audio_timestamps.first() {
    if *first_audio_pts > global_min_pts {
        let delay = first_audio_pts - global_min_pts;

        // Edit List 추가
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

실제 미디어 데이터를 담습니다.

**구조:**

```bash
Size (4 bytes)
Type: 'mdat'
Data (Size - 8 bytes)
  [비디오 프레임 1]
  [비디오 프레임 2]
  ...
  [오디오 프레임 1]
  [오디오 프레임 2]
  ...
```

**특징:**

- 가장 큰 box (보통 파일의 대부분)
- 순수 데이터만 포함
- stco가 이 데이터를 가리킴

**코드에서 생성:**

```rust
// 1. 데이터 준비
let mut mdat_data = Vec::new();

// 비디오 데이터
for sample in &video_samples {
    mdat_data.extend_from_slice(sample);
}
let video_data_end = mdat_data.len();

// 오디오 데이터
for sample in audio_samples {
    mdat_data.extend_from_slice(sample);
}

// 2. mdat box 생성
let mdat_size = 8 + mdat_data.len();
mp4_buffer.extend_from_slice(&(mdat_size as u32).to_be_bytes());
mp4_buffer.extend_from_slice(b"mdat");
mp4_buffer.extend_from_slice(&mdat_data);
```

## 3. MP4 생성 흐름

### 3.1 전체 프로세스

```bash
1. 비디오/오디오 데이터 준비
   ├─ Annex B → AVCC 변환 (비디오)
   └─ ADTS 제거 (오디오)
   ↓
2. mdat 데이터 조립
   ├─ 비디오 프레임들
   └─ 오디오 프레임들
   ↓
3. 오프셋 계산
   ├─ ftyp 크기
   ├─ moov 크기 (임시 계산)
   ├─ mdat 헤더
   └─ 각 청크 오프셋
   ↓
4. moov 박스 생성
   ├─ mvhd
   ├─ trak (비디오)
   │   └─ stbl (stsd, stts, stsc, stsz, stco, ctts)
   └─ trak (오디오)
       └─ stbl
   ↓
5. 최종 파일 조립
   [ftyp][moov][mdat]
```

### 3.2 코드에서의 메인 함수

```rust
pub fn create_mp4(media_data: MediaData) -> io::Result<Vec<u8>> {
    // 1. 비디오 데이터 준비
    let frames = split_into_frames(&media_data.video_stream);
    let mut video_samples = Vec::new();

    for frame in frames.iter() {
        let avcc_frame = convert_annexb_to_avcc(frame);
        video_samples.push(avcc_frame);
    }

    // 2. 오디오 데이터
    let audio_samples = &media_data.audio_frames;

    // 3. mdat 데이터 조립
    let mut mdat_data = Vec::new();
    for sample in &video_samples {
        mdat_data.extend_from_slice(sample);
    }
    let video_data_end = mdat_data.len();

    for sample in audio_samples {
        mdat_data.extend_from_slice(sample);
    }

    // 4. 오프셋 계산
    let ftyp_size = 28;
    let mdat_header_size = 8;

    // moov 크기 계산을 위한 임시 빌드
    let moov_box = build_moov(
        &media_data,
        &video_samples,
        audio_samples,
        ftyp_size,
        0,  // 임시
        mdat_header_size,
        video_data_end,
    )?;

    let moov_size = moov_box.len();

    // 5. 정확한 오프셋으로 moov 재생성
    let moov_box = build_moov(
        &media_data,
        &video_samples,
        audio_samples,
        ftyp_size,
        moov_size,
        mdat_header_size,
        video_data_end,
    )?;

    // 6. 최종 파일 조립
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

## 4. Timescale과 Duration

### 4.1 Timescale 개념

**Timescale**: 1초를 나타내는 단위 수

```bash
90kHz timescale = 1초 = 90000 units
48kHz timescale = 1초 = 48000 units
```

### 4.2 본 프로젝트의 Timescale 정책

**모든 트랙에 90kHz 통일:**

```bash
Movie timescale (mvhd): 90000 Hz
Video media timescale (mdhd): 90000 Hz
Audio media timescale (mdhd): 90000 Hz  ← 주의: 48000 아님!
```

**이유:** 호환성과 동기화 단순화

### 4.3 Duration 계산

**비디오 (30fps):**

```bash
1 프레임 = 1/30 초 = 90000/30 = 3000 units

700 프레임 = 700 × 3000 = 2,100,000 units
          = 2,100,000 / 90000 = 23.33초
```

**오디오 (AAC, 48kHz):**

```bash
1 프레임 = 1024 samples @ 48kHz
        = 1024/48000 초
        = 0.021333초
        = 0.021333 × 90000 = 1920 units

1095 프레임 = 1095 × 1920 = 2,102,400 units
           = 2,102,400 / 90000 = 23.36초
```

### 4.4 타임스탬프 정규화

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

// 모든 타임스탬프에서 global_min_pts를 빼서 0부터 시작하도록 정규화
```

## 5. 정리

### MP4의 계층 구조

```bash
MP4 File
├─ ftyp - 파일 타입
├─ moov - 메타데이터 컨테이너
│  ├─ mvhd - 영상 전체 정보
│  └─ trak - 각 트랙
│     ├─ tkhd - 트랙 정보
│     └─ mdia
│        ├─ mdhd - 미디어 timescale/duration
│        ├─ hdlr - 미디어 타입
│        └─ minf
│           ├─ vmhd/smhd - 비디오/오디오 헤더
│           └─ stbl - 샘플 테이블
│              ├─ stsd - 코덱 정보 (avc1/mp4a)
│              ├─ stts - 재생 시간
│              ├─ stsc - 청크 구조
│              ├─ stsz - 샘플 크기
│              ├─ stco - 데이터 위치
│              └─ ctts - 표시 시간 오프셋
└─ mdat - 실제 미디어 데이터
```

### 주요 개념

1. **Box 구조**: Size, Type, Data 를 재귀적으로 나타내는 구조
2. **Timescale**: 시간 표현 단위 (90kHz 통일)
3. **Sample**: 개별 프레임 (비디오) 또는 오디오 블록
4. **Chunk**: 연속된 샘플들의 그룹
5. **Offset**: 파일 내 절대 위치
6. **Duration**: Timescale 단위의 재생 시간
7. **PTS/DTS**: 표시/디코딩 시간
8. **Composition Offset**: PTS - DTS
