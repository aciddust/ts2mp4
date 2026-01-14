"""
Verify audio data positions in MP4 file

Usage:
  python verify_audio_data.py output.mp4
"""

import struct
import argparse

parser = argparse.ArgumentParser(description='Verify audio data positions and integrity in MP4 file')
parser.add_argument('input_file', help='Path to the MP4 file to analyze')
args = parser.parse_args()

with open(args.input_file, "rb") as f:
    data = f.read()

# Read audio sample sizes from stsz
stsz_positions = []
pos = 0
while True:
    pos = data.find(b"stsz", pos)
    if pos == -1:
        break
    stsz_positions.append(pos - 4)
    pos += 4

# Second stsz (audio)
audio_stsz_pos = stsz_positions[1]
stsz_data = data[audio_stsz_pos:]
sample_count = struct.unpack(">I", stsz_data[16:20])[0]

print(f"Audio sample count: {sample_count}")

# Read each sample size
sample_sizes = []
for i in range(sample_count):
    size = struct.unpack(">I", stsz_data[20 + i * 4 : 24 + i * 4])[0]
    sample_sizes.append(size)

# Read audio start position from stco
stco_positions = []
pos = 0
while True:
    pos = data.find(b"stco", pos)
    if pos == -1:
        break
    stco_positions.append(pos - 4)
    pos += 4

audio_stco_pos = stco_positions[1]
stco_data = data[audio_stco_pos:]
audio_start = struct.unpack(">I", stco_data[16:20])[0]

print(f"Audio data start offset (stco): {audio_start:,} (0x{audio_start:x})")

# 12 seconds = sample #562 (0.021333 seconds * 562 ≈ 12 seconds)
# if you need other sample index, change here
cumulative = 0
for i in range(562):
    cumulative += sample_sizes[i]

sample_n_offset = audio_start + cumulative
print(f"\n12 second position (sample #562 start):")
print(f"  Offset: {sample_n_offset:,} (0x{sample_n_offset:x})")
print(f"  File size: {len(data):,} bytes")
print(f"  Within file range? {sample_n_offset < len(data)}")
# Check if there's actual AAC data at that position
if sample_n_offset < len(data):
    sample_data = data[sample_n_offset : sample_n_offset + 10]
    print(f"  Data (hex): {sample_data.hex()}")
    # AAC frames usually start with 0xFF (ADTS header) or raw AAC

# Verify data every 100 samples
print(f"\nSample position verification (every 100 samples):")
cumulative = 0
for i in range(0, min(sample_count, 1000), 100):
    cumulative = sum(sample_sizes[:i])
    offset = audio_start + cumulative
    if offset < len(data):
        sample_data = data[offset : offset + 4]
        print(
            f"  Sample {i:4d}: offset={offset:10,} (0x{offset:08x}), size={sample_sizes[i]:4d}, data={sample_data.hex()}"
        )

# Check last sample
last_sample_offset = audio_start + sum(sample_sizes[:-1])
print(f"\nLast sample (#{sample_count}):")
print(f"  Offset: {last_sample_offset:,} (0x{last_sample_offset:x})")
print(f"  Size: {sample_sizes[-1]}")
print(f"  End position: {last_sample_offset + sample_sizes[-1]:,}")
if last_sample_offset < len(data):
    last_data = data[last_sample_offset : last_sample_offset + 4]
    print(f"  Data: {last_data.hex()}")
