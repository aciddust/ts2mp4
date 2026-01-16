# GIF 생성 기능

## 개요

TS 파일을 파싱하여 GIF 애니메이션을 생성하는 기능입니다.

## 구현 상태

### ✅ 완료된 부분

1. **GIF 인코더** (`src/gif_encoder.rs`)
   - GIF89a 포맷 완전 구현
   - RGB → 인덱스 컬러 변환
   - LZW 압축 알고리즘
   - 루프 설정 (무한 반복 지원)
   - 프레임 딜레이 조정 (FPS 기반)

2. **프레임 추출** (`src/frame_extractor.rs`)
   - I-frame 추출 로직
   - 플레이스홀더 프레임 생성 (테스트용)

3. **CLI 인터페이스**
   ```bash
   ts2mp4 gif <input.ts> <output.gif> [fps]
   ```

### ⚠️ 제한사항

**H.264 디코딩이 구현되지 않았습니다.**

현재는 플레이스홀더 그라디언트 프레임을 생성합니다. 실제 비디오 프레임을 GIF로 변환하려면 H.264 디코더가 필요합니다.

## 사용법

### CLI

```bash
# 기본 FPS (10fps)로 변환
./ts2mp4 gif input.ts output.gif

# 커스텀 FPS로 변환
./ts2mp4 gif input.ts output.gif 15
```

### 라이브러리

```rust
use ts2mp4::{convert_ts_to_gif, GifOptions};

let ts_data = std::fs::read("input.ts")?;

let options = GifOptions {
    fps: 15,
    loop_count: 0,    // 무한 반복
    max_colors: 256,  // 최대 색상 수
};

let gif_data = convert_ts_to_gif(&ts_data, Some(options))?;
std::fs::write("output.gif", gif_data)?;
```

## H.264 디코딩 구현 옵션

실제 비디오 프레임을 추출하려면 다음 중 하나를 선택해야 합니다:

### 옵션 1: 외부 프로세스 (ffmpeg)

```rust
use std::process::Command;
use std::fs;

// H.264 NAL units를 임시 파일로 저장
fs::write("temp.h264", nal_data)?;

// ffmpeg로 RGB 변환
Command::new("ffmpeg")
    .args(&[
        "-i", "temp.h264",
        "-f", "rawvideo",
        "-pix_fmt", "rgb24",
        "temp.rgb"
    ])
    .status()?;

// RGB 데이터 읽기
let rgb_data = fs::read("temp.rgb")?;
```

**장점**: 구현 간단, 검증된 디코더  
**단점**: ffmpeg 설치 필요, 파일 I/O 오버헤드

### 옵션 2: 크레이트 사용

```toml
[dependencies]
# 옵션 A: ffmpeg 바인딩
ffmpeg-next = "7.0"

# 옵션 B: OpenH264
openh264 = "0.4"

# 옵션 C: 순수 Rust 구현 (WIP)
dav1d-rs = "0.10"  # AV1용, H.264는 제한적
```

**장점**: Rust 네이티브 통합  
**단점**: 의존성 추가, 복잡도 증가

### 옵션 3: 저수준 직접 구현

H.264 스펙을 직접 구현 (권장하지 않음)

**장점**: 의존성 없음  
**단점**: 매우 복잡함 (수천 줄 코드), 버그 위험

## 구조

```
src/
├── frame_extractor.rs  # H.264 프레임 추출
├── gif_encoder.rs      # GIF 인코딩
├── lib.rs              # 공개 API
└── main.rs             # CLI
```

## GIF 포맷 세부사항

### 헤더 구조

```
GIF89a
Logical Screen Descriptor
Global Color Table (256 colors)
Netscape Extension (반복 설정)
```

### 프레임 구조

각 프레임마다:
```
Graphics Control Extension (딜레이 설정)
Image Descriptor
Image Data (LZW 압축)
```

### 색상 양자화

- RGB (24비트) → 인덱스 컬러 (8비트, 256색)
- 유클리드 거리로 가장 가까운 팔레트 색상 선택
- 균일 분포 팔레트 사용

### LZW 압축

- 초기 코드 크기: 8비트
- 동적 코드 크기 증가 (9-12비트)
- 딕셔너리 자동 리셋 (4096 엔트리)

## 향후 개선사항

1. **H.264 디코더 통합**
   - ffmpeg-next 크레이트 사용 고려
   - 또는 외부 프로세스 방식

2. **성능 최적화**
   - 멀티스레딩 (프레임별 병렬 처리)
   - SIMD 최적화 (색상 변환)

3. **고급 기능**
   - 프레임 리사이징
   - 색상 팔레트 최적화 (median cut, octree)
   - 디더링 옵션
   - 투명도 지원

4. **WASM 지원**
   - 웹 브라우저에서 직접 변환

## 참고 자료

- [GIF89a Specification](https://www.w3.org/Graphics/GIF/spec-gif89a.txt)
- [LZW Compression](https://en.wikipedia.org/wiki/Lempel%E2%80%93Ziv%E2%80%93Welch)
- [H.264/AVC Specification](https://www.itu.int/rec/T-REC-H.264)
