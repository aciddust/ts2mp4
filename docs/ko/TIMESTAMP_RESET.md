# 타임스탬프 리셋 기능

## 개요

라이브 스트리밍에서 다운로드한 TS 세그먼트를 MP4로 변환할 때, 원본 방송의 절대 타임스탬프가 유지되어 재생 문제가 발생할 수 있습니다. 이 기능은 FFmpeg의 `-avoid_negative_ts make_zero` 옵션과 동일한 방식으로 타임스탬프를 0부터 시작하도록 리셋합니다.

## 문제 상황 예시

m3u8 스트리밍에서 다운로드한 세그먼트를 변환했을 때:

```bash
ffprobe -show_format output.mp4
```

```json
{
  "format": {
    "start_time": "38735.100000",  // 약 10시간 47분
    "duration": "38740.100000",     // 약 10시간 47분
    "size": "5191797"               // 5.2MB (실제로는 5초 분량)
  }
}
```

- 플레이어가 start_time부터 재생하려고 시도하여 1초만 재생되거나 재생 불가
- 실제 데이터는 5초 분량이지만 메타데이터는 10시간으로 표시

## 사용 방법

### JavaScript/TypeScript (웹 환경)

#### 기존 함수 (타임스탬프 유지)

```javascript
import init, { convert_ts_to_mp4_wasm } from './pkg/ts2mp4.js';

await init();

const tsData = await fetch('input.ts').then(r => r.arrayBuffer());
const mp4Data = convert_ts_to_mp4_wasm(new Uint8Array(tsData));
```

#### 새 함수 (타임스탬프 리셋) ⭐

```javascript
import init, { convert_ts_to_mp4_reset_timestamps_wasm } from './pkg/ts2mp4.js';

await init();

const tsData = await fetch('input.ts').then(r => r.arrayBuffer());
const mp4Data = convert_ts_to_mp4_reset_timestamps_wasm(new Uint8Array(tsData));

// 파일로 저장
const blob = new Blob([mp4Data], { type: 'video/mp4' });
const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = 'output.mp4';
a.click();
```

### Rust (네이티브 환경)

```rust
use ts2mp4::convert_ts_to_mp4_with_options;
use std::fs;

fn main() -> std::io::Result<()> {
    let ts_data = fs::read("input.ts")?;

    // 타임스탬프 리셋
    let mp4_data = convert_ts_to_mp4_with_options(&ts_data, true)?;

    fs::write("output.mp4", mp4_data)?;
    Ok(())
}
```

## API 레퍼런스

### WASM API

#### `convert_ts_to_mp4_wasm(ts_data: Uint8Array): Uint8Array`

기존 함수 - 타임스탬프를 원본대로 유지합니다.

**파라미터:**

- `ts_data`: MPEG-TS 바이너리 데이터

**반환값:**

- MP4 바이너리 데이터

#### `convert_ts_to_mp4_reset_timestamps_wasm(ts_data: Uint8Array): Uint8Array`

새 함수 - 타임스탬프를 0부터 시작하도록 리셋합니다 (FFmpeg의 `-avoid_negative_ts make_zero`와 동일).

**파라미터:**

- `ts_data`: MPEG-TS 바이너리 데이터

**반환값:**

- MP4 바이너리 데이터 (타임스탬프 리셋됨)

#### `reset_mp4_timestamps_wasm(mp4_data: Uint8Array): Uint8Array`

새 함수 - MP4 파일의 타임스탬프를 0부터 시작하도록 리셋합니다.

**파라미터:**

- `mp4_data`: MP4 바이너리 데이터

**반환값:**

- MP4 바이너리 데이터 (타임스탬프 리셋됨)

**사용 예시:**

```javascript
import init, { reset_mp4_timestamps_wasm } from './pkg/ts2mp4.js';

await init();

const mp4Data = await fetch('broken.mp4').then(r => r.arrayBuffer());
const fixedMp4 = reset_mp4_timestamps_wasm(new Uint8Array(mp4Data));
```

### Rust API

#### `convert_ts_to_mp4(ts_data: &[u8]) -> io::Result<Vec<u8>>`

기존 함수 - 타임스탬프를 원본대로 유지합니다.

#### `convert_ts_to_mp4_with_options(ts_data: &[u8], reset_timestamps: bool) -> io::Result<Vec<u8>>`

옵션을 받는 함수 - `reset_timestamps`로 동작을 제어합니다.

**파라미터:**

- `ts_data`: MPEG-TS 바이너리 데이터
- `reset_timestamps`: `true`일 경우 타임스탬프를 0부터 시작하도록 리셋

**반환값:**

- MP4 바이너리 데이터

#### `reset_mp4_timestamps(mp4_data: &[u8]) -> io::Result<Vec<u8>>`

MP4 파일의 타임스탬프를 리셋하는 함수.

**파라미터:**

- `mp4_data`: MP4 바이너리 데이터

**반환값:**

- 타임스탬프가 리셋된 MP4 바이너리 데이터

**사용 예시:**

```rust
use ts2mp4::reset_mp4_timestamps;
use std::fs;

fn main() -> std::io::Result<()> {
    let mp4_data = fs::read("broken.mp4")?;
    let fixed_mp4 = reset_mp4_timestamps(&mp4_data)?;
    fs::write("fixed.mp4", fixed_mp4)?;
    Ok(())
}
```

## 내부 동작 원리

1. **PTS/DTS 최소값 계산**: 비디오와 오디오 스트림의 모든 타임스탬프 중 최소값을 찾습니다.

2. **타임스탬프 정규화**: 모든 타임스탬프에서 최소값을 빼서 0부터 시작하도록 조정합니다.

3. **Edit List 조정**: 오디오-비디오 동기화를 위한 Edit List도 함께 조정됩니다.

```rust
let global_min_pts = if reset_timestamps {
    // 비디오와 오디오의 최소 PTS 중 더 작은 값
    match (video_min_pts, audio_min_pts) {
        (Some(v), Some(a)) => v.min(a),
        (Some(v), None) => v,
        (None, Some(a)) => a,
        (None, None) => 0,
    }
} else {
    0  // 기존 동작 유지
};
```

## 검증 방법

### 변환 전

```bash
ffprobe -show_format input.mp4
```

```bash
start_time: 38735.100000
duration: 38740.100000
```

### 변환 후

```bash
ffprobe -show_format output.mp4
```

```bash
start_time: 0.000000
duration: 5.000000
```
