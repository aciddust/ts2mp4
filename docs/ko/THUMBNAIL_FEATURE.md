# 썸네일 추출 기능

## 개요

ts2mp4 라이브러리에 TS 파일과 MP4 파일에서 썸네일을 추출하는 기능이 추가되었습니다.

## 주요 특징

### 구현 내용

1. **TS 파일 썸네일 추출**
   - 첫 번째 I-frame (IDR 프레임) 추출
   - SPS/PPS NAL units 포함
   - Annex B 형식으로 출력

2. **MP4 파일 썸네일 추출**
   - 첫 번째 키프레임 추출
   - AVCC 형식을 Annex B로 변환
   - MP4 박스 구조 파싱

3. **외부 의존성 없음**
   - image 크레이트 불필요
   - 순수 Rust 구현
   - 기존 의존성 유지

### 출력 형식

- Raw H.264 NAL units (Annex B 형식)
- ffmpeg으로 JPEG/PNG 등으로 변환 가능
- H.264 디코더에서 직접 사용 가능

## 사용 방법

### CLI

```bash
# TS 파일에서 썸네일 추출
ts2mp4 thumbnail-ts input.ts thumbnail.h264

# MP4 파일에서 썸네일 추출
ts2mp4 thumbnail-mp4 input.mp4 thumbnail.h264

# 이미지로 변환
ffmpeg -i thumbnail.h264 -frames:v 1 thumbnail.jpg
```

### Rust API

```rust
use ts2mp4::{extract_thumbnail_from_ts, extract_thumbnail_from_mp4};
use std::fs;

// TS에서 추출
let ts_data = fs::read("input.ts")?;
let thumbnail = extract_thumbnail_from_ts(&ts_data)?;
fs::write("thumbnail.h264", thumbnail)?;

// MP4에서 추출
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

// TS에서 추출
const tsData = new Uint8Array(await tsFile.arrayBuffer());
const thumbnail = extract_thumbnail_from_ts_wasm(tsData);

// MP4에서 추출
const mp4Data = new Uint8Array(await mp4File.arrayBuffer());
const thumbnail = extract_thumbnail_from_mp4_wasm(mp4Data);
```

## 기술 세부사항

### TS 파싱

- NAL unit 탐색 (0x00 0x00 0x00 0x01 또는 0x00 0x00 0x01)
- IDR 프레임 (NAL type 5) 감지
- SPS/PPS와 함께 완전한 프레임 구성

### MP4 파싱

- mdat 박스에서 첫 번째 샘플 추출
- stsz 박스에서 샘플 크기 확인
- AVCC → Annex B 변환 (length prefix → start code)

### 변환 과정

```bash
AVCC 형식:
[4바이트 길이][NAL unit][4바이트 길이][NAL unit]...

Annex B 형식:
[0x00 0x00 0x00 0x01][NAL unit][0x00 0x00 0x00 0x01][NAL unit]...
```

## 파일 구조

```bash
src/
  thumbnail.rs           # 썸네일 추출 모듈
  lib.rs                # Public API 노출
  main.rs               # CLI 명령어 구현

examples/
  extract_thumbnail.rs  # 사용 예제

web/
  thumbnail.html        # 웹 데모 페이지

docs/
  ko/
    USAGE.md           # 한국어 사용 가이드
    DEV_GUIDE.md       # 한국어 개발 가이드
  en/
    USAGE.md           # 영어 사용 가이드
    DEV_GUIDE.md       # 영어 개발 가이드
```

## 테스트

```bash
# 단위 테스트
cargo test

# 릴리스 빌드
cargo build --release

# WASM 빌드
wasm-pack build --target web

# 예제 실행
cargo run --example extract_thumbnail -- input.ts thumbnail.h264
```

## 활용 예시

### 1. 비디오 썸네일 생성

```bash
ts2mp4 thumbnail-ts video.ts thumb.h264
ffmpeg -i thumb.h264 -vf scale=320:240 thumb_small.jpg
```

### 2. 웹 애플리케이션

- 브라우저에서 직접 썸네일 추출
- 서버 업로드 없이 클라이언트 측 처리
- 프리뷰 생성

### 3. 비디오 처리 파이프라인

- 자동 썸네일 생성
- 비디오 인덱싱
- 미리보기 이미지 생성

## 성능

- **메모리**: 추가 메모리 할당 최소화
- **속도**: 첫 번째 I-frame만 추출하여 빠른 처리
- **크기**: 외부 의존성 없어 바이너리 크기 증가 없음

## 제한사항

- 첫 번째 키프레임만 추출 (다중 썸네일 미지원)
- H.264 코덱만 지원
- 이미지 디코딩은 별도 도구 필요 (ffmpeg 등)
