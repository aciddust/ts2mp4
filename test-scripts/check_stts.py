"""
Usage:

Basic usage:
python test-scripts/check_stts.py output.mp4
"""

import struct
import argparse

parser = argparse.ArgumentParser(description='Check STTS (Time-to-Sample) box information in MP4 file')
parser.add_argument('input_file', help='Path to the MP4 file to analyze')
args = parser.parse_args()

with open(args.input_file, "rb") as f:
    data = f.read()

# stts 박스 찾기
stts_positions = []
pos = 0
while True:
    pos = data.find(b"stts", pos)
    if pos == -1:
        break
    stts_positions.append(pos - 4)
    pos += 4

print(f"Found {len(stts_positions)} stts boxes\n")

for i, stts_pos in enumerate(stts_positions):
    size = struct.unpack(">I", data[stts_pos : stts_pos + 4])[0]
    stts_data = data[stts_pos : stts_pos + size]
    entry_count = struct.unpack(">I", stts_data[12:16])[0]

    track_type = "Video" if i == 0 else "Audio"
    print(f"{track_type} stts box:")
    print(f"  Entry count: {entry_count}")

    for j in range(min(3, entry_count)):
        sample_count = struct.unpack(">I", stts_data[16 + j * 8 : 20 + j * 8])[0]
        sample_delta = struct.unpack(">I", stts_data[20 + j * 8 : 24 + j * 8])[0]
        print(
            f"  Entry {j+1}: sample_count={sample_count}, sample_delta={sample_delta}"
        )

        # 실제 시간 계산
        if track_type == "Video":
            # 비디오는 90kHz 타임베이스
            duration_sec = (sample_count * sample_delta) / 90000.0
            print(
                f"    -> {duration_sec:.3f} seconds (delta={sample_delta/90000.0:.6f}s)"
            )
        else:
            # 오디오도 90kHz 타임베이스
            duration_sec = (sample_count * sample_delta) / 90000.0
            print(
                f"    -> {duration_sec:.3f} seconds (delta={sample_delta/90000.0:.6f}s)"
            )
            # AAC는 1024 samples @ 48kHz = 0.021333s
            print(f"    (Expected AAC: 1024/48000 = 0.021333s = 1920 in 90kHz)")

    print()

# 오디오 duration 계산
if len(stts_positions) >= 2:
    audio_stts_pos = stts_positions[1]
    stts_data = data[audio_stts_pos:]
    sample_count = struct.unpack(">I", stts_data[16:20])[0]
    sample_delta = struct.unpack(">I", stts_data[20:24])[0]

    total_duration_90k = sample_count * sample_delta
    total_duration_sec = total_duration_90k / 90000.0

    print(f"Total audio duration calculated from stts:")
    print(
        f"  {sample_count} samples × {sample_delta} delta = {total_duration_90k} (90kHz units)"
    )
    print(f"  = {total_duration_sec:.3f} seconds")
    print(f"\nExpected: 1095 × 1024 / 48000 = {1095 * 1024 / 48000:.3f} seconds")
