# MP4 파서 구현 완료 🎉

## 구현된 기능

### 1. 핵심 MP4 파서 ([src/mp4_parser.rs](../../src/mp4_parser.rs))

#### 기본 박스 파싱

- `Mp4Box` - MP4 박스 구조체
- `Mp4File` - 파싱된 MP4 파일 정보
- `Mp4Reader` - 바이트 읽기 헬퍼
- `parse_mp4()` - 최상위 박스 파싱 (ftyp, moov, mdat)
- `parse_container_box()` - 컨테이너 박스 내부 파싱

#### 타임스탬프 관련 박스 파싱

- `parse_mvhd()` - Movie Header (전역 타임스탬프)
- `parse_mdhd()` - Media Header (트랙별 타임스탬프)
- `parse_tkhd()` - Track Header
- `parse_stts()` - Decoding Time to Sample
- `parse_ctts()` - Composition Time to Sample

#### 샘플 정보 파싱

- `parse_stsz()` - Sample Size
- `parse_stco()` - Chunk Offset (32-bit)
- `parse_co64()` - Chunk Offset (64-bit)
- `parse_stsc()` - Sample to Chunk
- `parse_hdlr()` - Handler Reference (미디어 타입)

#### 타임스탬프 리셋 로직

- `reset_mp4_timestamps()` - 메인 함수
- `reset_moov_timestamps()` - moov 박스 처리
- `reset_mvhd()` - Movie Header 리셋
- `reset_trak_timestamps()` - 트랙별 리셋
- `reset_tkhd()` - Track Header 리셋
- `reset_mdia_timestamps()` - Media 박스 리셋
- `reset_mdhd()` - Media Header 리셋
- **Edit List (edts) 자동 제거** - 타임스탬프 리셋 시 불필요

### 2. 통합 및 인터페이스

#### Rust 라이브러리 ([src/lib.rs](../../src/lib.rs))

```rust
// 공개 API
pub use mp4_parser::reset_mp4_timestamps;

// 기존 함수도 유지
pub fn convert_ts_to_mp4_with_options(ts_data: &[u8], reset_timestamps: bool) -> io::Result<Vec<u8>>;
```

#### WASM 인터페이스

```javascript
// 기존 함수
convert_ts_to_mp4_wasm(ts_data: Uint8Array): Uint8Array
convert_ts_to_mp4_reset_timestamps_wasm(ts_data: Uint8Array): Uint8Array

// 새로 추가된 함수
reset_mp4_timestamps_wasm(mp4_data: Uint8Array): Uint8Array
```

#### CLI ([src/main.rs](../../src/main.rs))

```bash
# TS 파일 타임스탬프 리셋
ts2mp4 convert -i input.ts -o output.mp4 -r

# MP4 파일 타임스탬프 리셋 (새로 추가!)
ts2mp4 convert -i input.mp4 -o output.mp4 -r
```

## 동작 방식

### MP4 구조 파싱

```bash
MP4 File
├── ftyp (File Type)
├── moov (Movie)
│   ├── mvhd (Movie Header) ← 타임스탬프 리셋
│   ├── trak (Track)
│   │   ├── tkhd (Track Header) ← 타임스탬프 리셋
│   │   ├── edts (Edit List) ← 제거됨
│   │   └── mdia (Media)
│   │       ├── mdhd (Media Header) ← 타임스탬프 리셋
│   │       ├── hdlr (Handler)
│   │       └── minf (Media Info)
│   │           └── stbl (Sample Table)
│   │               ├── stts (Time to Sample)
│   │               ├── ctts (Composition Offset)
│   │               ├── stsz (Sample Size)
│   │               ├── stco/co64 (Chunk Offset)
│   │               └── stsc (Sample to Chunk)
│   └── ...
└── mdat (Media Data)
```

### 타임스탬프 리셋 과정

1. **MP4 파싱**: 모든 박스를 메모리에 로드
2. **타임스탬프 박스 수정**:
   - `mvhd`: creation_time, modification_time → 0
   - `tkhd`: creation_time, modification_time → 0
   - `mdhd`: creation_time, modification_time → 0
3. **Edit List 제거**: edts 박스 삭제 (불필요해짐)
4. **재구성**: ftyp + 수정된 moov + 원본 mdat

### 특징

**비파괴적**: 미디어 데이터(mdat)는 수정하지 않음
**빠름**: 메타데이터만 수정하므로 매우 빠름
**안전함**: 원본 샘플 데이터는 그대로 유지
**호환성**: FFmpeg `-avoid_negative_ts make_zero`와 동일한 결과

## 사용 예시

### CLI

```bash
# 문제 있는 MP4 파일
ffprobe broken.mp4
# start_time: 38735.100000

# 타임스탬프 리셋
ts2mp4 convert -i broken.mp4 -o fixed.mp4 --reset-timestamps

# 결과 확인
ffprobe fixed.mp4
# start_time: 0.000000
```

### WASM (JavaScript)

```javascript
import init, { reset_mp4_timestamps_wasm } from './pkg/ts2mp4.js';

await init();

// MP4 파일 로드
const mp4Data = await fetch('broken.mp4').then(r => r.arrayBuffer());

// 타임스탬프 리셋
const fixedMp4 = reset_mp4_timestamps_wasm(new Uint8Array(mp4Data));

// 다운로드
const blob = new Blob([fixedMp4], { type: 'video/mp4' });
const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = 'fixed.mp4';
a.click();
```

### Rust

```rust
use ts2mp4::reset_mp4_timestamps;
use std::fs;

fn main() -> std::io::Result<()> {
    let mp4_data = fs::read("broken.mp4")?;
    let fixed_mp4 = reset_mp4_timestamps(&mp4_data)?;
    fs::write("fixed.mp4", fixed_mp4)?;
    println!("Timestamps reset successfully!");
    Ok(())
}
```

## 테스트

### 단위 테스트

```bash
cargo test
```

```rust
#[test]
fn test_mp4_reader() {
    let data = vec![0x00, 0x00, 0x00, 0x01, 0x12, 0x34, 0x56, 0x78];
    let mut reader = Mp4Reader::new(&data);
    assert_eq!(reader.read_u32().unwrap(), 1);
    assert_eq!(reader.read_u32().unwrap(), 0x12345678);
}

#[test]
fn test_full_box_header() {
    let data = vec![0x01, 0x00, 0x00, 0x03];
    let (version, flags) = read_full_box_header(&data).unwrap();
    assert_eq!(version, 1);
    assert_eq!(flags, 3);
}
```

### 통합 테스트

```bash
# TS → MP4 변환 (타임스탬프 리셋)
ts2mp4 convert -i test.ts -o output1.mp4 -r
ffprobe output1.mp4  # start_time = 0

# MP4 → MP4 타임스탬프 리셋
ts2mp4 convert -i broken.mp4 -o output2.mp4 -r
ffprobe output2.mp4  # start_time = 0

# 플레이어에서 재생 테스트
mpv output1.mp4
mpv output2.mp4
```
