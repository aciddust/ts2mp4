"""
Usage:

Basic usage:
python test-scripts/check_audio_position.py output.mp4

Specify sample index (default is 562):
python test-scripts/check_audio_position.py output.mp4 --sample-index 1000

Specify audio start position:
python test-scripts/check_audio_position.py output.mp4 --audio-start 9361436 --sample-index 562
"""

import struct
import argparse

parser = argparse.ArgumentParser(description='Check audio sample positions in MP4 file')
parser.add_argument('input_file', help='Path to the MP4 file to analyze')
parser.add_argument('--audio-start', type=int, help='Audio data start position (auto-detected if not specified)')
parser.add_argument('--sample-index', type=int, default=562, help='Sample index to check (default: 562)')
args = parser.parse_args()

with open(args.input_file, "rb") as f:
    data = f.read()

# read audio sample size from stsz box
stsz_positions = []
pos = 0
while True:
    pos = data.find(b"stsz", pos)
    if pos == -1:
        break
    stsz_positions.append(pos - 4)
    pos += 4

# second stsz (audio)
audio_stsz_pos = stsz_positions[1]
stsz_data = data[audio_stsz_pos:]
sample_count = struct.unpack(">I", stsz_data[16:20])[0]

print(f"Total audio samples: {sample_count}")

# read each sample size
sample_sizes = []
for i in range(sample_count):
    size = struct.unpack(">I", stsz_data[20 + i * 4 : 24 + i * 4])[0]
    sample_sizes.append(size)

# Get audio start position
if args.audio_start:
    audio_start = args.audio_start
else:
    # Auto-detect from stco (chunk offset) - second stco is audio
    stco_positions = []
    pos = 0
    while True:
        pos = data.find(b"stco", pos)
        if pos == -1:
            break
        stco_positions.append(pos - 4)
        pos += 4

    if len(stco_positions) > 1:
        audio_stco_pos = stco_positions[1]
        stco_data = data[audio_stco_pos:]
        audio_start = struct.unpack(">I", stco_data[16:20])[0]
        print(f"Auto-detected audio start: {audio_start:,} (0x{audio_start:x})")
    else:
        print("Error: Could not auto-detect audio start position. Please specify --audio-start")
        exit(1)

# Calculate cumulative size up to specified sample
cumulative = 0
for i in range(min(args.sample_index, sample_count)):
    cumulative += sample_sizes[i]

print(f"\nSample #{args.sample_index} position:")
print(f"  Audio start: {audio_start:,} (0x{audio_start:x})")
print(f"  Cumulative size of {args.sample_index} samples: {cumulative:,} bytes")
print(
    f"  Position of sample #{args.sample_index}: {audio_start + cumulative:,} (0x{audio_start + cumulative:x})"
)
print(f"  File size: {len(data):,} bytes")
print(f"  Is sample #{args.sample_index} within file range? {audio_start + cumulative < len(data)}")

# Check mdat size
mdat_pos = data.find(b"mdat") - 4
mdat_size = struct.unpack(">I", data[mdat_pos : mdat_pos + 4])[0]
mdat_end = mdat_pos + mdat_size

print(f"\nmdat box:")
print(f"  Start: {mdat_pos:,} (0x{mdat_pos:x})")
print(f"  Size: {mdat_size:,}")
print(f"  End: {mdat_end:,} (0x{mdat_end:x})")
print(f"  Is sample #{args.sample_index} within mdat range? {audio_start + cumulative < mdat_end}")

# Total audio data size
total_audio = sum(sample_sizes)
print(f"\nTotal audio data size: {total_audio:,} bytes")
print(
    f"Audio end position: {audio_start + total_audio:,} (0x{audio_start + total_audio:x})"
)
print(f"Is audio data within mdat range? {audio_start + total_audio <= mdat_end}")