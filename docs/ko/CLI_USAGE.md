# CLI 사용 가이드

## 설치

### 소스에서 빌드

```bash
git clone https://github.com/aciddust/ts2mp4.git
cd ts2mp4
cargo build --release
```

실행 파일: `./target/release/ts2mp4`

### 설치 (선택사항)

```bash
cargo install --path .
```

이후 어디서든 `ts2mp4` 명령어 사용 가능

## 명령어

### 1. 파일 변환 (Convert)

TS 또는 MP4 파일을 MP4로 변환합니다. 입력 파일 형식은 자동으로 감지됩니다.

#### 기본 변환 (타임스탬프 유지)

```bash
ts2mp4 convert -i input.ts -o output.mp4
```

#### 타임스탬프 리셋

라이브 스트리밍 세그먼트 처리에 권장됩니다.

```bash
ts2mp4 convert -i input.ts -o output.mp4 --reset-timestamps
```

또는 짧게:

```bash
ts2mp4 convert -i input.ts -o output.mp4 -r
```

### 2. 썸네일 추출

#### TS 파일에서 추출

```bash
ts2mp4 thumbnail-ts -i input.ts -o thumbnail.jpg
```

#### MP4 파일에서 추출

```bash
ts2mp4 thumbnail-mp4 -i input.mp4 -o thumbnail.jpg
```

## 상세 옵션

### convert 명령어

```bash
ts2mp4 convert [OPTIONS] --input <INPUT> --output <OUTPUT>
```

**옵션:**
- `-i, --input <INPUT>` - 입력 파일 경로 (TS 또는 MP4)
- `-o, --output <OUTPUT>` - 출력 MP4 파일 경로
- `-r, --reset-timestamps` - 타임스탬프를 0부터 시작하도록 리셋 (FFmpeg의 `-avoid_negative_ts make_zero`와 동일)

### thumbnail-ts 명령어

```bash
ts2mp4 thumbnail-ts --input <INPUT> --output <OUTPUT>
```

**옵션:**
- `-i, --input <INPUT>` - 입력 TS 파일 경로
- `-o, --output <OUTPUT>` - 출력 JPEG 파일 경로

### thumbnail-mp4 명령어

```bash
ts2mp4 thumbnail-mp4 --input <INPUT> --output <OUTPUT>
```

**옵션:**
- `-i, --input <INPUT>` - 입력 MP4 파일 경로
- `-o, --output <OUTPUT>` - 출력 JPEG 파일 경로

## 사용 예시

### 라이브 스트리밍 다운로드 후 변환

```bash
# 1. m3u8에서 세그먼트 다운로드 (예시)
ffmpeg -i "https://example.com/live.m3u8" -c copy segments.ts

# 2. 타임스탬프 리셋하여 MP4로 변환
ts2mp4 convert -i segments.ts -o output.mp4 --reset-timestamps

# 3. 결과 확인
ffprobe -show_format output.mp4
```

### 배치 처리

```bash
# 여러 TS 파일을 일괄 변환
for file in *.ts; do
    ts2mp4 convert -i "$file" -o "${file%.ts}.mp4" -r
done
```

### 자동 파일 형식 감지

CLI는 파일 내용을 분석하여 자동으로 TS 또는 MP4를 감지합니다.

```bash
# TS 파일로 감지됨
ts2mp4 convert -i video.ts -o output.mp4 -r

# MP4 파일로 감지됨
ts2mp4 convert -i video.mp4 -o output.mp4
# (MP4에 타임스탬프 리셋은 아직 미지원)
```

## 출력 예시

### 정상 변환

```
Input: input.ts
Output: output.mp4
Timestamp reset: enabled
Detected: MPEG-TS format
Conversion complete!
```

### 에러 발생

```
Input: corrupted.ts
Output: output.mp4
Detected: MPEG-TS format
Error: No valid TS sync byte found
```

## 문제 해결

### "MP4 timestamp reset is not yet implemented"

~~MP4 파일에 대한 타임스탬프 리셋은 아직 지원되지 않습니다. 원본 TS 파일을 사용하세요.~~

**업데이트: MP4 타임스탬프 리셋이 이제 지원됩니다!**

```bash
# MP4 → MP4 타임스탬프 리셋 지원
ts2mp4 convert -i video.mp4 -o fixed.mp4 -r

# TS → MP4 타임스탬프 리셋도 계속 지원
ts2mp4 convert -i original.ts -o fixed.mp4 -r
```

### "Unknown file format"

파일이 TS 또는 MP4 형식이 아닙니다. 파일을 확인하세요.

```bash
# 파일 형식 확인
file input.ts
```

### 타임스탬프가 여전히 이상함

FFmpeg로 검증:

```bash
ffprobe -v error -show_format -show_streams output.mp4
```

`start_time`이 0에 가까운지 확인하세요.
