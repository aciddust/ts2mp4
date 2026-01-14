# Why 90kHz?

When studying MPEG-2 TS, the number **90kHz** might seem quite random.

Normally, we use units like `ms (milliseconds)` or `ns (nanoseconds)` for time.

The reason this number was chosen is a combination of **1. Mathematical reasons (frame rate compatibility)** and **2. Hardware reasons (system clock)**.

## TL;DR

* 90kHz serves as a base unit that can satisfy various broadcasting standard frame rates without error
  * It's the least common multiple that satisfies most frame rates
* Used for synchronizing video output timing

---

## 1. Mathematical Reason: "Least Common Multiple" of All Frame Rates

The broadcasting/video industry has various frame rate (FPS) standards:

* **24 FPS** (Cinema)
* **25 FPS** (European PAL broadcasting)
* **30 FPS** (US/Korean NTSC broadcasting)
* **50 FPS, 60 FPS** (High-definition broadcasting)

What happens if we use **milliseconds (1/1000 second)** as the time unit?

* For 30 FPS, time per frame: ms

This results in an **infinite decimal**. As decimals accumulate over 1 or 2 hours of broadcasting, **cumulative errors cause sync drift issues**.

But what if we divide based on **90,000 (90kHz)**?

| Frame Rate (FPS) | Frames per Second | 90,000 Unit Conversion (ticks per frame) | Result |
| --- | --- | --- | --- |
| **24** (Cinema) | 24 | $90,000 \div 24$ | **3,750** (exact division) |
| **25** (PAL) | 25 | $90,000 \div 25$ | **3,600** (exact division) |
| **30** (NTSC) | 30 | $90,000 \div 30$ | **3,000** (exact division) |
| **60** | 60 | $90,000 \div 60$ | **1,500** (exact division) |

**Conclusion:**

Using 90,000 allows us to represent major broadcasting standard frame rates as **exact integers without decimals**. Since computers perform integer calculations much faster and more accurately than floating-point calculations, this value was adopted as the standard.

---

### 2. Hardware Reason: 27MHz System Clock

The master clock governing the entire MPEG-2 system (STC, System Time Clock) is **27MHz (27,000,000 Hz)**.

* **Why 27MHz?**: This engineering standard value emerged from finding a common multiple of legacy analog color TV signal frequencies and digital audio sampling rates.

However, using 27MHz directly for PTS/DTS timestamps would cause numbers to grow too quickly, consuming too much data (overhead). So the MPEG standards committee decided:

> "Let's use the master clock (27MHz) **divided by 300** as the basic timestamp unit."

This makes implementation very simple, as hardware circuits only need to increment the timestamp by 1 every time they count 300 clock signals.

---

## 3. How Does 90kHz Relate to Video Formats Like TS and MP4?

* When to display on screen
* When to start decoding
* Frame calculation

Used for frame output timing synchronization

<details>
<summary>Click for more detailed information</summary>

### 3.1 Usage in MPEG-2 TS

**PES (Packetized Elementary Stream) Header:**

```bash
[PES Header]
├─ PTS (Presentation Time Stamp) - 33 bits, 90kHz unit
│   → Determines when to display on screen
│
└─ DTS (Decoding Time Stamp) - 33 bits, 90kHz unit
    → Determines when to start decoding (only when B-frames exist)
```

**Binary Example:**

```rust
// Extract PTS from PES header (5 bytes, 33 bits)
let pts = ((pes_header[9] as u64 & 0x0E) << 29)
        | ((pes_header[10] as u64) << 22)
        | ((pes_header[11] as u64 & 0xFE) << 14)
        | ((pes_header[12] as u64) << 7)
        | ((pes_header[13] as u64) >> 1);

// Stored in 90kHz units
// Example: pts = 270000 → 3 seconds (270000 / 90000 = 3.0)
```

**PCR (Program Clock Reference):**

Based on 27MHz but related to 90kHz:

```bash
PCR = PCR_base × 300 + PCR_extension
      ^^^^^^^^         ^^^^^^^^^^^^^^
      (90kHz)          (27MHz remainder)
```

### 3.2 Usage in MP4

**Track Header (tkhd box):**

```bash
duration: 32/64 bits
→ Based on movie timescale (usually 90000)
```

**Media Header (mdhd box):**

```bash
timescale: 32 bits  → Usually 90000 (90kHz)
duration: 32/64 bits → Based on timescale
```

**Time-to-Sample Table (stts box):**

```bash
sample_count: Number of samples
sample_delta: Time to next sample in 90kHz units

Example (30 FPS video):
- sample_delta = 3000 (90000 / 30)
- Increases by 3000 ticks per frame
```

**Composition Time to Sample (ctts box):**

```bash
sample_count: Number of samples
sample_offset: PTS - DTS difference (90kHz units)

Example (with B-frames):
- I-frame: offset = 6000 (delayed by 2 B-frames)
- P-frame: offset = 0
- B-frame: offset = -3000 (display time advanced)
```

**Practical Code Usage:**

```rust
// Set timescale when generating MP4
const TIMESCALE: u32 = 90000;  // 90kHz

// Calculate video frame delta (30 FPS)
let video_delta = TIMESCALE / 30;  // 3000

// Calculate audio frame delta (48kHz, 1024 samples)
let audio_delta = (TIMESCALE as u64 * 1024) / 48000;  // 1920

// Calculate duration (seconds → ticks)
let duration_ticks = (duration_seconds * TIMESCALE as f64) as u64;
```

### 3.3 Conversion Considerations

**When Converting TS → MP4:**

Since TS PTS/DTS are already in 90kHz units, setting MP4's timescale to 90000 allows **direct use of values**.

```rust
// PTS read from TS PES
let ts_pts: u64 = 8100000;  // 90 seconds

// When MP4 timescale = 90000
let mp4_timestamp = ts_pts;  // Can use directly!

// If MP4 timescale is different, conversion needed
let mp4_timescale = 48000;  // If set to 48kHz
let converted_timestamp = (ts_pts * mp4_timescale) / 90000;
```

**Frame Interval Validation:**

```rust
// Verify it's 30 FPS video
let expected_delta = 90000 / 30;  // 3000
assert_eq!(actual_delta, expected_delta);

// AAC 48kHz, 1024 samples
let expected_audio_delta = (90000 * 1024) / 48000;  // 1920
assert_eq!(audio_delta, expected_audio_delta);
```

</details>

---

### 4. Summary

**Why 90kHz:**

1. **Accuracy:** To represent various video frame intervals (24, 25, 30, 60 FPS, etc.) as **exact integers without decimal errors**.
2. **Convenience:** It's the MPEG system's master clock **27MHz divided by an integer (300)**, making hardware design easier.
3. **Result:** Therefore, when handling PTS values in code, you must divide by `PTS / 90000.0` to get the familiar **"seconds"** unit.

**Code Usage Example:**

```rust
let pts_value: u64 = 8_100_000; // Value read from TS packet
let seconds = pts_value as f64 / 90_000.0;

println!("Playback time: {} seconds", seconds); // "Playback time: 90 seconds"
```
