"""
Usage:

Basic usage:
python test-scripts/check_duration.py output.mp4
"""
import struct
import argparse

parser = argparse.ArgumentParser(description='Check duration information in MP4 file')
parser.add_argument('input_file', help='Path to the MP4 file to analyze')
args = parser.parse_args()

with open(args.input_file, "rb") as f:
    data = f.read()

print("=" * 60)
print(f"Duration box information from {args.input_file}")
print("=" * 60)

# mvhd (movie header)
mvhd_pos = data.find(b"mvhd") - 4
if mvhd_pos >= 0:
    mvhd_data = data[mvhd_pos:]
    timescale = struct.unpack(">I", mvhd_data[20:24])[0]
    duration = struct.unpack(">I", mvhd_data[24:28])[0]

    print(f"\n[MVHD - Movie Header]")
    print(f"Timescale: {timescale} (Hz)")
    print(f"Duration: {duration} (units)")
    print(f"Duration (seconds): {duration / timescale:.3f}")

# tkhd (track header) - 2개 있을 것
tkhd_positions = []
pos = 0
while True:
    pos = data.find(b"tkhd", pos)
    if pos == -1:
        break
    tkhd_positions.append(pos - 4)
    pos += 4

print(f"\n[TKHD - Track Headers] (found {len(tkhd_positions)})")
for i, tkhd_pos in enumerate(tkhd_positions):
    tkhd_data = data[tkhd_pos:]
    version = tkhd_data[8]

    if version == 0:
        # version 0: 32-bit duration
        duration = struct.unpack(">I", tkhd_data[28:32])[0]
    else:
        # version 1: 64-bit duration
        duration = struct.unpack(">Q", tkhd_data[36:44])[0]

    track_type = "Video" if i == 0 else "Audio"
    print(f"\n{track_type} Track:")
    print(f"  Version: {version}")
    print(f"  Duration: {duration} (in movie timescale)")
    print(
        f"  Duration (seconds): {duration / timescale:.3f}"
        if "timescale" in locals()
        else ""
    )

# mdhd (media header) - 각 트랙의 미디어 헤더
mdhd_positions = []
pos = 0
while True:
    pos = data.find(b"mdhd", pos)
    if pos == -1:
        break
    mdhd_positions.append(pos - 4)
    pos += 4

print(f"\n[MDHD - Media Headers] (found {len(mdhd_positions)})")
for i, mdhd_pos in enumerate(mdhd_positions):
    mdhd_data = data[mdhd_pos:]
    version = mdhd_data[8]

    if version == 0:
        timescale = struct.unpack(">I", mdhd_data[20:24])[0]
        duration = struct.unpack(">I", mdhd_data[24:28])[0]
    else:
        timescale = struct.unpack(">I", mdhd_data[28:32])[0]
        duration = struct.unpack(">Q", mdhd_data[32:40])[0]

    track_type = "Video" if i == 0 else "Audio"
    print(f"\n{track_type} Media:")
    print(f"  Version: {version}")
    print(f"  Timescale: {timescale} (Hz)")
    print(f"  Duration: {duration} (units)")
    print(f"  Duration (seconds): {duration / timescale:.3f}")

# stts로 계산한 duration
stts_positions = []
pos = 0
while True:
    pos = data.find(b"stts", pos)
    if pos == -1:
        break
    stts_positions.append(pos - 4)
    pos += 4

print(f"\n[STTS-based Duration Calculation]")
for i, stts_pos in enumerate(stts_positions):
    stts_data = data[stts_pos:]
    sample_count = struct.unpack(">I", stts_data[16:20])[0]
    sample_delta = struct.unpack(">I", stts_data[20:24])[0]

    total_duration = sample_count * sample_delta
    track_type = "Video" if i == 0 else "Audio"

    print(f"\n{track_type}:")
    print(f"  Sample count: {sample_count}")
    print(f"  Sample delta: {sample_delta}")
    print(f"  Total duration: {total_duration} (90kHz units)")
    print(f"  Duration (seconds): {total_duration / 90000:.3f}")
