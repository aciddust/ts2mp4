# NOTE

## Challenges During Development

Major issues resolved during project development:

### 1. Audio Cutting Off Midway Through Video

- **Cause**: STTS sample_delta set to 1024 (sample count)
- **Solution**: Changed to 1920 (reflecting 90kHz timescale)

### 2. ffplay Has Audio Info but QuickTime Cuts Off Before Midpoint

- **Cause**: MDHD timescale set to 48000
- **Solution**: Unified to 90000

### 3. ffplay Has Audio Info but QuickTime Cuts Off at 12 Seconds (Follow-up to Issue #2)

- **Cause**: TKHD duration calculated with wrong timescale
- **Solution**: Calculate based on movie timescale (90kHz)

Most audio playback issues were resolved by **unifying the timescale**
