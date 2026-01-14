# 개발 가이드

TS to MP4 변환기 개발 및 디버깅 가이드입니다.

## 빌드 방법

### Debug 빌드

```bash
cargo build
```

### Release 빌드 (최적화됨)

```bash
cargo build --release
```

실행 파일 위치:

- Debug: `target/debug/ts2mp4`
- Release: `target/release/ts2mp4`

### WebAssembly 빌드

```bash
# wasm-pack 설치 (최초 1회)
cargo install wasm-pack

# WASM 빌드
wasm-pack build --target web
```

## 사용 방법

### 기본 사용법

```bash
./target/release/ts2mp4 <입력.ts> <출력.mp4>

# 예시
./target/release/ts2mp4 input.ts output.mp4
```

### 출력 예시

```bash
Converting input.ts to output.mp4
Found PIDs - Video: 256, Audio: 257
Total audio frames collected: 1095
Total video frames collected: 700
Audio PTS range: 104160 - 2179680 (1.16 - 24.22 sec)
Video PTS range: 108000 - 2205000 (1.20 - 24.50 sec)
Conversion completed successfully!
```

## 테스트 및 검증

### ffprobe로 메타데이터 확인

```bash
# 기본 정보
ffprobe output.mp4

# 스트림 정보만
ffprobe -v error -show_streams output.mp4

# Duration만 확인
ffprobe -v error -show_entries stream=codec_name,duration -of default=nw=1 output.mp4

# 자세한 포맷 정보
ffprobe -v error -show_format -show_streams output.mp4
```

**예상 출력:**

```bash
Stream #0:0(und): Video: h264 (Main) (avc1 / 0x31637661), yuv420p(progressive),
  1280x720 [SAR 1:1 DAR 16:9], 3203 kb/s, 30 fps, 30 tbr, 90k tbn (default)
Stream #0:1(und): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp,
  192 kb/s (default)
```

### ffplay로 재생 테스트

```bash
# 기본 재생
ffplay output.mp4

# 자동 종료 (재생 완료 후)
ffplay -autoexit output.mp4

# 에러만 출력
ffplay -autoexit -loglevel error output.mp4
```

### 패킷 정보 확인

```bash
# 모든 패킷 출력
ffprobe -v error -show_packets output.mp4

# 오디오 패킷만
ffprobe -v error -select_streams a:0 -show_packets output.mp4

# 비디오 패킷만
ffprobe -v error -select_streams v:0 -show_packets output.mp4

# 패킷 개수 확인
ffprobe -v error -count_packets -show_entries stream=nb_read_packets output.mp4
```

## 디버깅 도구 (Python 스크립트)

프로젝트에는 MP4 구조를 분석하는 여러 Python 스크립트가 포함되어 있습니다.

### 1. analyze_mp4.py - MP4 구조 분석

**용도**: MP4 파일의 기본 박스 구조 확인

```bash
python3 test-scripts/analyze_mp4.py output.mp4
```

**출력 정보:**

- STSC (Sample-to-Chunk): 청크 구조
- STCO (Chunk Offset): 데이터 위치
- STSZ (Sample Size): 각 샘플 크기
- MDAT: 실제 미디어 데이터 위치

**사용 시점:**

- 청크 구조가 올바른지 확인
- 파일 크기와 오프셋 검증
- 샘플 개수 확인

### 2. check_all_durations.py - Duration 검증

**용도**: 모든 duration 관련 박스 확인

```bash
python3 test-scripts/check_all_durations.py
```

**출력 정보:**

- MVHD (Movie Header): 전체 영상 duration
- TKHD (Track Header): 각 트랙 duration
- MDHD (Media Header): 미디어별 timescale과 duration

**사용 시점:**

- QuickTime에서 재생 시간이 이상할 때
- Duration 불일치 문제 진단
- Timescale 설정 확인

**예상 출력:**

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

### 3. check_stts.py - Time-to-Sample 검증

**용도**: STTS 박스의 sample_delta 확인

```bash
python3 test-scripts/check_stts.py
```

**출력 정보:**

- 각 트랙의 sample_count와 sample_delta
- 계산된 총 duration
- AAC 프레임 duration 검증

**사용 시점:**

- 오디오가 중간에 끊길 때
- Duration 계산이 틀렸을 때
- Timescale 변환 확인

**주의사항:**

- 비디오 sample_delta: 3000 (30fps, 90kHz)
- 오디오 sample_delta: 1920 (1024 samples @ 48kHz in 90kHz)

### 4. verify_audio_data.py - 오디오 데이터 위치 검증

**용도**: 실제 파일에서 오디오 데이터 위치 확인

```bash
python3 test-scripts/verify_audio_data.py
```

**사용 시점:**

- 오디오가 특정 시점에 끊길 때
- 데이터가 파일에 실제로 있는지 확인
- STCO 오프셋이 올바른지 검증

### 5. debug_9sec.py - 특정 시점 디버깅

**용도**: 9초 지점의 상세 분석 (임의의 시점으로 수정 가능)

```bash
python3 test-scripts/debug_9sec.py
```

**사용 시점:**

- 특정 시점에서 재생이 멈출 때
- 데이터 오프셋 계산 검증

## MP4 구조 이해

### 기본 박스 구조

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

### 중요한 Timescale 개념

모든 duration과 timestamp는 **90kHz timescale**을 사용합니다:

- **Movie timescale (mvhd)**: 90000 Hz
- **Video media timescale (mdhd)**: 90000 Hz
- **Audio media timescale (mdhd)**: 90000 Hz (48000이 아님!)

**계산 예시:**

- 비디오 1프레임 (30fps): 90000 / 30 = 3000 units
- 오디오 1프레임 (AAC 1024 samples @ 48kHz): 1024 / 48000 * 90000 = 1920 units

### STTS (Time-to-Sample) 설정

```rust
// 비디오
sample_count: 700
sample_delta: 3000  // 90000 / 30fps

// 오디오
sample_count: 1095
sample_delta: 1920  // (1024 samples / 48000 Hz) * 90000
```

### STSC (Sample-to-Chunk) 최적화

단일 청크 방식 사용 (호환성 향상):

```rust
// 비디오
first_chunk: 1
samples_per_chunk: 700  // 모든 샘플을 1개 청크에

// 오디오
first_chunk: 1
samples_per_chunk: 1095  // 모든 샘플을 1개 청크에
```

**이유**: 많은 작은 청크를 만들면 QuickTime 등 일부 플레이어에서 재생 문제 발생

## 일반적인 문제 해결

### 1. 오디오가 중간에 끊김

**증상**: ffplay는 정상, QuickTime은 12초에서 끊김

**원인**: STTS sample_delta 또는 MDHD timescale 불일치

**해결**:

```bash
# 1. Duration 확인
python3 test-scripts/check_all_durations.py

# 2. STTS 확인
python3 test-scripts/check_stts.py

# 3. 오디오 sample_delta가 1920인지 확인
# 4. 오디오 mdhd timescale이 90000인지 확인
```

### 2. 재생 시간이 실제와 다름

**증상**: ffprobe는 23초인데 실제 재생은 12초

**원인**: TKHD duration이 잘못된 timescale로 계산됨

**해결**:

- TKHD duration은 **movie timescale (90kHz)**로 계산
- MDHD duration은 **media timescale (90kHz)**로 계산
- 둘 다 같은 값이어야 함

### 3. QuickTime에서 재생 안 됨

**증상**: ffplay는 되는데 QuickTime은 안 됨

**가능한 원인**:

1. avcC 박스 누락 또는 잘못됨
2. esds 박스 누락 (오디오)
3. Duration 불일치
4. 박스 크기 오류

**해결**:

```bash
# 1. 구조 확인
python3 test-scripts/analyze_mp4.py output.mp4

# 2. ffprobe로 코덱 확인
ffprobe -v error -show_streams output.mp4

# 3. 박스 크기 수동 확인
xxd output.mp4 | head -100
```

### 4. 오디오 sync 문제

**증상**: 오디오/비디오 싱크가 안 맞음

**원인**: Global minimum PTS 정규화 문제

**해결**:

- 코드에서 `global_min_pts` 계산 확인
- Edit List (edts) 박스 추가 고려
- 오디오 시작 시간이 비디오보다 늦으면 edts로 조정

## 코드 수정 시 체크리스트

박스 구조를 수정할 때 확인할 사항:

- [ ] 박스 크기(size) 필드가 정확한가?
- [ ] Timescale이 일관되는가? (모두 90kHz)
- [ ] Sample_delta 계산이 올바른가?
- [ ] Duration이 모든 헤더에서 일치하는가?
- [ ] STCO 오프셋이 실제 데이터 위치와 맞는가?
- [ ] STSZ에 모든 샘플 크기가 기록되었는가?
- [ ] STSC가 실제 청크 구조와 일치하는가?

## hexdump로 직접 확인

```bash
# MP4 헤더 확인 (처음 100줄)
xxd output.mp4 | head -100

# 특정 박스 찾기
xxd output.mp4 | grep "mvhd"
xxd output.mp4 | grep "stts"

# mdat 위치 찾기
xxd output.mp4 | grep -n "mdat"
```

## 추가 참고 자료

- [ISO/IEC 14496-12:2022](https://www.iso.org/standard/83102.html) - ISO base media file format
- [ISO/IEC 14496-14](https://www.iso.org/standard/79110.html) - MP4 파일 포맷
- [ISO/IEC 14496-15:2024](https://www.iso.org/standard/89118.html) - AVC 파일 포맷 (NAL 유닛 구조)
- [MP4RA](https://mp4ra.org/) - MP4 등록 기관

## 디버깅 팁

1. **항상 ffprobe 먼저 실행**: 기본 구조가 올바른지 확인
2. **Python 스크립트 활용**: 박스별 상세 정보 확인
3. **ffplay로 테스트**: 실제 재생 가능 여부 확인
4. **QuickTime으로 최종 검증**: 가장 엄격한 플레이어
5. **단계적 디버깅**: 비디오만 먼저 → 오디오 추가 → 동기화

## 성능 최적화

- Release 빌드 사용 (`--release`)
- 큰 파일은 청크 단위로 처리 고려
- WASM에서는 Web Worker 사용 권장
- 불필요한 박스 생성 최소화
