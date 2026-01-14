"""
Debug MP4 file at a specific time position

Usage:
  python debug_n_sec.py output.mp4
  python debug_n_sec.py output.mp4 --time 12.5
"""

import struct
import argparse

parser = argparse.ArgumentParser(description='Debug MP4 file structure at a specific time position')
parser.add_argument('input_file', help='Path to the MP4 file to analyze')
parser.add_argument('--time', type=float, default=9.0, help='Time position in seconds to analyze (default: 9.0)')
args = parser.parse_args()

with open(args.input_file, "rb") as f:
    data = f.read()

# Collect all relevant box information
print("=" * 60)
print(f"Detailed MP4 Structure Analysis at {args.time}s")
print("=" * 60)

# stsz (샘플 크기)
stsz_positions = []
pos = 0
while True:
    pos = data.find(b"stsz", pos)
    if pos == -1:
        break
    stsz_positions.append(pos - 4)
    pos += 4

audio_stsz_pos = stsz_positions[1]
stsz_data = data[audio_stsz_pos:]
sample_count = struct.unpack(">I", stsz_data[16:20])[0]

sample_sizes = []
for i in range(sample_count):
    size = struct.unpack(">I", stsz_data[20 + i * 4 : 24 + i * 4])[0]
    sample_sizes.append(size)

print(f"\n[Audio Sample Information]")
print(f"Total samples: {sample_count}")
print(f"First 10 sample sizes: {sample_sizes[:10]}")

# stco (청크 오프셋)
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
chunk_count = struct.unpack(">I", stco_data[12:16])[0]
audio_chunk_offset = struct.unpack(">I", stco_data[16:20])[0]

print(f"\n[STCO - Chunk Offset]")
print(f"Chunk count: {chunk_count}")
print(f"Audio chunk start: {audio_chunk_offset:,} (0x{audio_chunk_offset:x})")

# stsc (샘플-투-청크)
stsc_positions = []
pos = 0
while True:
    pos = data.find(b"stsc", pos)
    if pos == -1:
        break
    stsc_positions.append(pos - 4)
    pos += 4

audio_stsc_pos = stsc_positions[1]
stsc_data = data[audio_stsc_pos:]
entry_count = struct.unpack(">I", stsc_data[12:16])[0]
first_chunk = struct.unpack(">I", stsc_data[16:20])[0]
samples_per_chunk = struct.unpack(">I", stsc_data[20:24])[0]

print(f"\n[STSC - Sample-to-Chunk]")
print(f"Entry count: {entry_count}")
print(f"First chunk: {first_chunk}")
print(f"Samples per chunk: {samples_per_chunk}")

# Calculate sample index at specified time (AAC frame = 1024 samples @ 48kHz = 0.021333s)
AAC_FRAME_DURATION = 1024 / 48000  # ~0.021333 seconds
samples_at_time = int(args.time / AAC_FRAME_DURATION)
print(f"\n[Analysis at {args.time}s]")
print(f"{args.time}s ≈ sample #{samples_at_time} (frame duration = {AAC_FRAME_DURATION:.6f}s)")

# Cumulative size up to the target sample
if samples_at_time > sample_count:
    print(f"Warning: Requested time {args.time}s exceeds available samples ({sample_count})")
    samples_at_time = sample_count

cumulative = sum(sample_sizes[:samples_at_time])
sample_offset = audio_chunk_offset + cumulative

print(f"Cumulative size of samples 0~{samples_at_time}: {cumulative:,} bytes")
print(f"Sample #{samples_at_time} offset: {sample_offset:,} (0x{sample_offset:x})")
print(f"File size: {len(data):,} bytes")
print(f"Within file range? {sample_offset < len(data)}")

if sample_offset < len(data):
    sample_data = data[sample_offset : sample_offset + 10]
    print(f"Data (hex): {sample_data.hex()}")

# mdat 박스 확인
mdat_pos = data.find(b"mdat") - 4
mdat_size = struct.unpack(">I", data[mdat_pos : mdat_pos + 4])[0]
mdat_data_start = mdat_pos + 8
mdat_data_end = mdat_pos + mdat_size

print(f"\n[MDAT Box]")
print(f"Position: {mdat_pos:,} (0x{mdat_pos:x})")
print(f"Size: {mdat_size:,} bytes")
print(f"Data start: {mdat_data_start:,} (0x{mdat_data_start:x})")
print(f"Data end: {mdat_data_end:,} (0x{mdat_data_end:x})")

# Check if audio chunk is within mdat range
print(
    f"\nAudio chunk within mdat range? {mdat_data_start <= audio_chunk_offset < mdat_data_end}"
)

# 총 오디오 데이터 크기
total_audio_size = sum(sample_sizes)
audio_end = audio_chunk_offset + total_audio_size

print(f"\n[Total Audio Data]")
print(f"Start: {audio_chunk_offset:,} (0x{audio_chunk_offset:x})")
print(f"Size: {total_audio_size:,} bytes")
print(f"End: {audio_end:,} (0x{audio_end:x})")
print(f"Matches mdat end? {audio_end == mdat_data_end}")
print(f"Matches file end? {audio_end == len(data)}")

# 비디오 데이터 확인
video_stco_pos = stco_positions[0]
video_stco_data = data[video_stco_pos:]
video_chunk_offset = struct.unpack(">I", video_stco_data[16:20])[0]

video_stsz_pos = stsz_positions[0]
video_stsz_data = data[video_stsz_pos:]
video_sample_count = struct.unpack(">I", video_stsz_data[16:20])[0]

video_sizes = []
for i in range(video_sample_count):
    size = struct.unpack(">I", video_stsz_data[20 + i * 4 : 24 + i * 4])[0]
    video_sizes.append(size)

total_video_size = sum(video_sizes)
video_end = video_chunk_offset + total_video_size

print(f"\n[Video Data]")
print(f"Start: {video_chunk_offset:,} (0x{video_chunk_offset:x})")
print(f"Size: {total_video_size:,} bytes")
print(f"End: {video_end:,} (0x{video_end:x})")
print(f"Matches audio start? {video_end == audio_chunk_offset}")
