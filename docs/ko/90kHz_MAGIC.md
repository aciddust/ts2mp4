# 왜 90kHz?

MPEG-2 TS를 공부하다 보면 **90kHz**라는 숫자가 정말 뜬금없어 보일 수 있습니다.

보통 시간은 `ms(밀리초)`나 `ns(나노초)`를 쓰니까요.

이 숫자가 선택된 이유는 크게 **1. 수학적인 이유(프레임률 호환성)** 와 **2. 하드웨어적인 이유(시스템 클럭)** 두 가지가 결합되어 있기 때문입니다.

## TL;DR

* 90kHz 를 기준으로 여러 방송 표준 frame rate 를 오차 없이 만족시킬 수 있음
  * 대부분의 frame rate를 만족시키는 최소공배수이기때문
* 영상 출력시점 동기화에 사용되고있음

---

## 1. 수학적 이유: 모든 frame rate의 "최소공배수"

방송/영상 업계에는 다양한 프레임률(FPS) 표준이 존재합니다.

* **24 FPS** (영화)
* **25 FPS** (유럽 PAL 방송)
* **30 FPS** (미국/한국 NTSC 방송)
* **50 FPS, 60 FPS** (고화질 방송)

만약 시간을 **밀리초(1/1000초)** 단위로 쓴다면 어떻게 될까요?

* 30 FPS의 경우 1프레임당 시간: ms

이렇게 **무한 소수**가 발생합니다. 소수점이 계속 쌓이면 1시간, 2시간 방송이 지속될 때 **오차가 누적되어 싱크가 틀어지는 문제(Drift)** 가 생깁니다.

하지만 **90,000(90kHz)** 을 기준으로 나누면 어떻게 될까요?

| 프레임률 (FPS) | 1초당 프레임 수 | 90,000 단위 환산 (1 프레임당 tick) | 결과 |
| --- | --- | --- | --- |
| **24** (영화) | 24 | $90,000 \div 24$ | **3,750** (딱 떨어짐) |
| **25** (PAL) | 25 | $90,000 \div 25$ | **3,600** (딱 떨어짐) |
| **30** (NTSC) | 30 | $90,000 \div 30$ | **3,000** (딱 떨어짐) |
| **60** | 60 | $90,000 \div 60$ | **1,500** (딱 떨어짐) |

**결론:**

90,000을 사용하면 주요 방송 표준 프레임률들을 **소수점 없이 정수(Integer)로 딱 떨어지게** 표현할 수 있습니다. 컴퓨터는 소수점 계산보다 정수 계산이 훨씬 빠르고 정확하기 때문에 이 값을 표준으로 잡은 것입니다.

---

### 2. 하드웨어적 이유: 27MHz 시스템 클럭

MPEG-2 시스템 전체를 관장하는 마스터 클럭(STC, System Time Clock)은 **27MHz (27,000,000 Hz)** 입니다.

* **왜 27MHz인가?**: 과거 아날로그 컬러 TV 신호 주파수들과 디지털 오디오 샘플링 속도 등의 공배수를 찾다 보니 나온 공학적인 표준값입니다.

하지만 PTS/DTS 타임스탬프에 27MHz를 그대로 쓰면 숫자가 너무 빨리 커져서 데이터를 많이 차지합니다(오버헤드). 그래서 MPEG 표준 위원회는 이렇게 결정했습니다.

> "마스터 클럭(27MHz)을 **300으로 나눈 값**을 기본 타임스탬프 단위로 쓰자."

이렇게 하면 하드웨어 회로에서 클럭 신호를 300번 셀 때마다 타임스탬프를 1씩 올리면 되므로 구현이 매우 간단해집니다.

---

## 3. 그래서 TS와 MP4 같은 영상에서 90kHz 는 어떤 연관성이 있는가?

* 언제 화면에 표시할지
* 언제 디코딩을 시작할지
* 프레임 계산

프레임 출력 시점 동기화를 위한 계산에 사용됨

<details>
<summary>조금 더 자세한 정보가 알고싶다면 클릭</summary>

### 3.1 MPEG-2 TS에서의 사용

**PES (Packetized Elementary Stream) 헤더:**

```bash
[PES Header]
├─ PTS (Presentation Time Stamp) - 33 bits, 90kHz 단위
│   → 언제 화면에 표시할지 결정
│
└─ DTS (Decoding Time Stamp) - 33 bits, 90kHz 단위
    → 언제 디코딩을 시작할지 결정 (B-frame 있을 때만)
```

**실제 바이너리 예시:**

```rust
// PES 헤더에서 PTS 추출 (5바이트, 33비트)
let pts = ((pes_header[9] as u64 & 0x0E) << 29)
        | ((pes_header[10] as u64) << 22)
        | ((pes_header[11] as u64 & 0xFE) << 14)
        | ((pes_header[12] as u64) << 7)
        | ((pes_header[13] as u64) >> 1);

// 90kHz 단위로 저장됨
// 예: pts = 270000 → 3초 (270000 / 90000 = 3.0)
```

**PCR (Program Clock Reference):**

27MHz 기반이지만 90kHz와 관련:

```bash
PCR = PCR_base × 300 + PCR_extension
      ^^^^^^^^         ^^^^^^^^^^^^^^
      (90kHz)          (27MHz의 나머지)
```

### 3.2 MP4에서의 사용

**Track Header (tkhd box):**

```bash
duration: 32/64 bits
→ movie timescale 기준 (보통 90000)
```

**Media Header (mdhd box):**

```bash
timescale: 32 bits  → 대부분 90000 (90kHz)
duration: 32/64 bits → timescale 기준
```

**Time-to-Sample Table (stts box):**

```bash
sample_count: 각 샘플의 개수
sample_delta: 90kHz 단위로 다음 샘플까지의 시간

예시 (30 FPS 비디오):
- sample_delta = 3000 (90000 / 30)
- 매 프레임마다 3000 tick씩 증가
```

**Composition Time to Sample (ctts box):**

```bash
sample_count: 샘플 개수
sample_offset: PTS - DTS 차이 (90kHz 단위)

예시 (B-frame 있을 때):
- I-frame: offset = 6000 (2 B-frame만큼 지연)
- P-frame: offset = 0
- B-frame: offset = -3000 (표시 시간이 앞당겨짐)
```

**실제 코드에서의 활용:**

```rust
// MP4 생성 시 timescale 설정
const TIMESCALE: u32 = 90000;  // 90kHz

// 비디오 프레임 delta 계산 (30 FPS)
let video_delta = TIMESCALE / 30;  // 3000

// 오디오 프레임 delta 계산 (48kHz, 1024 samples)
let audio_delta = (TIMESCALE as u64 * 1024) / 48000;  // 1920

// duration 계산 (초 → tick)
let duration_ticks = (duration_seconds * TIMESCALE as f64) as u64;
```

### 3.3 변환 시 주의사항

**TS → MP4 변환 시:**

TS의 PTS/DTS는 이미 90kHz 단위이므로, MP4의 timescale을 90000으로 설정하면 **값을 그대로 사용**할 수 있습니다.

```rust
// TS PES에서 읽은 PTS
let ts_pts: u64 = 8100000;  // 90초

// MP4 timescale = 90000인 경우
let mp4_timestamp = ts_pts;  // 그대로 사용 가능!

// 만약 MP4 timescale이 다르다면 변환 필요
let mp4_timescale = 48000;  // 48kHz로 설정했다면
let converted_timestamp = (ts_pts * mp4_timescale) / 90000;
```

**프레임 간격 검증:**

```rust
// 30 FPS 비디오가 맞는지 확인
let expected_delta = 90000 / 30;  // 3000
assert_eq!(actual_delta, expected_delta);

// AAC 48kHz, 1024 samples
let expected_audio_delta = (90000 * 1024) / 48000;  // 1920
assert_eq!(audio_delta, expected_audio_delta);
```

</details>

---

### 4. 전체 요약

**90kHz가 나온 이유:**

1. **정확성:** 24, 25, 30, 60 FPS 등 다양한 영상 프레임 간격을 **소수점 오차 없이 정수**로 표현하기 위해.
2. **편의성:** MPEG 시스템의 마스터 클럭인 **27MHz를 정수(300)로 나눈 값**이라서 하드웨어 설계가 편함.
3. **결과:** 따라서 코드에서 PTS 값을 다룰 때는 `PTS / 90000.0`을 해야 우리가 아는 **"초(second)"** 단위가 됩니다.

**코드 활용 예시:**

```rust
let pts_value: u64 = 8_100_000; // TS 패킷에서 읽은 값
let seconds = pts_value as f64 / 90_000.0;

println!("재생 시간: {}초", seconds); // "재생 시간: 90초"
```
