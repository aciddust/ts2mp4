"""
Usage:

Basic usage:
python test-scripts/check_all_durations.py output.mp4
"""
import struct
import argparse

# Parse command line arguments
parser = argparse.ArgumentParser(description='Check MP4 duration information')
parser.add_argument('input_file', help='Path to the MP4 file to analyze')
args = parser.parse_args()

with open(args.input_file, "rb") as f:
    data = f.read()

print("=" * 60)
print(f"Duration header information from {args.input_file}")
print("=" * 60)

# mvhd
mvhd_pos = data.find(b"mvhd")
mvhd_ts = struct.unpack(">I", data[mvhd_pos + 12 : mvhd_pos + 16])[0]
mvhd_dur = struct.unpack(">I", data[mvhd_pos + 16 : mvhd_pos + 20])[0]
print(f"\nMVHD (Movie Header):")
print(f"  Timescale: {mvhd_ts} Hz")
print(f"  Duration: {mvhd_dur} units")
print(f"  Duration: {mvhd_dur/mvhd_ts:.3f} seconds")

# tkhd
pos = 0
i = 0
print(f"\nTKHD (Track Headers):")
while True:
    pos = data.find(b"tkhd", pos)
    if pos == -1:
        break
    version = data[pos + 8]
    if version == 0:
        dur = struct.unpack(">I", data[pos + 28 : pos + 32])[0]
    else:
        dur = struct.unpack(">Q", data[pos + 36 : pos + 44])[0]

    track = "Video" if i == 0 else "Audio"
    print(f"  {track} Track:")
    print(f"    Version: {version}")
    print(f"    Duration: {dur} (in movie timescale {mvhd_ts} Hz)")
    print(f"    Duration: {dur/mvhd_ts:.3f} seconds")
    pos += 4
    i += 1

# mdhd
pos = 0
i = 0
print(f"\nMDHD (Media Headers):")
while True:
    pos = data.find(b"mdhd", pos)
    if pos == -1:
        break
    ts = struct.unpack(">I", data[pos + 12 : pos + 16])[0]
    dur = struct.unpack(">I", data[pos + 16 : pos + 20])[0]
    track = "Video" if i == 0 else "Audio"
    print(f"  {track} Media:")
    print(f"    Timescale: {ts} Hz")
    print(f"    Duration: {dur} units")
    print(f"    Duration: {dur/ts:.3f} seconds")
    pos += 4
    i += 1

# 예상값 계산
print(f'\n{"="*60}')
print("예상 값:")
print(f"  Video: 700 frames × 3000 delta / 90000 = {700*3000/90000:.3f} sec")
print(f"  Audio: 1095 frames × 1920 delta / 90000 = {1095*1920/90000:.3f} sec")
print(f"  Movie: max(video, audio) = {max(700*3000, 1095*1920)/90000:.3f} sec")
