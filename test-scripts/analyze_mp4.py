"""
Usage:

Basic usage:
python test-scripts/analyze_mp4.py output.mp4
"""

import struct
import argparse


def analyze_mp4(filename):
    with open(filename, "rb") as f:
        data = f.read()

    print("=" * 60)
    print(f"Analyzing: {filename}")
    print("=" * 60)

    # stsc 박스 찾기 (오디오/비디오 트랙용)
    stsc_positions = []
    pos = 0
    while True:
        pos = data.find(b"stsc", pos)
        if pos == -1:
            break
        stsc_positions.append(pos - 4)
        pos += 4

    print(f"\n[STSC] Found {len(stsc_positions)} stsc boxes")
    for i, stsc_pos in enumerate(stsc_positions):
        size = struct.unpack(">I", data[stsc_pos : stsc_pos + 4])[0]
        stsc_data = data[stsc_pos : stsc_pos + size]
        entry_count = struct.unpack(">I", stsc_data[12:16])[0]
        print(f"\nstsc #{i+1} at offset {stsc_pos}: size={size}, entries={entry_count}")
        for j in range(min(3, entry_count)):
            first_chunk = struct.unpack(">I", stsc_data[16 + j * 12 : 20 + j * 12])[0]
            samples_per_chunk = struct.unpack(
                ">I", stsc_data[20 + j * 12 : 24 + j * 12]
            )[0]
            desc_index = struct.unpack(">I", stsc_data[24 + j * 12 : 28 + j * 12])[0]
            print(
                f"  Entry {j+1}: first_chunk={first_chunk}, samples_per_chunk={samples_per_chunk}, desc_index={desc_index}"
            )
        if entry_count > 3:
            print(f"  ... and {entry_count-3} more entries")

    # stco 박스 찾기
    stco_positions = []
    pos = 0
    while True:
        pos = data.find(b"stco", pos)
        if pos == -1:
            break
        stco_positions.append(pos - 4)
        pos += 4

    print(f"\n\n[STCO] Found {len(stco_positions)} stco boxes")
    for i, stco_pos in enumerate(stco_positions):
        size = struct.unpack(">I", data[stco_pos : stco_pos + 4])[0]
        stco_data = data[stco_pos : stco_pos + size]
        entry_count = struct.unpack(">I", stco_data[12:16])[0]
        print(f"\nstco #{i+1} at offset {stco_pos}: size={size}, chunks={entry_count}")
        for j in range(min(5, entry_count)):
            offset = struct.unpack(">I", stco_data[16 + j * 4 : 20 + j * 4])[0]
            print(f"  Chunk {j+1} offset: {offset} (0x{offset:x})")
        if entry_count > 5:
            print(f"  ... and {entry_count-5} more chunks")

    # stsz 박스 찾기
    stsz_positions = []
    pos = 0
    while True:
        pos = data.find(b"stsz", pos)
        if pos == -1:
            break
        stsz_positions.append(pos - 4)
        pos += 4

    print(f"\n\n[STSZ] Found {len(stsz_positions)} stsz boxes")
    for i, stsz_pos in enumerate(stsz_positions):
        size = struct.unpack(">I", data[stsz_pos : stsz_pos + 4])[0]
        stsz_data = data[stsz_pos : stsz_pos + size]
        sample_size = struct.unpack(">I", stsz_data[12:16])[0]
        sample_count = struct.unpack(">I", stsz_data[16:20])[0]
        print(f"\nstsz #{i+1} at offset {stsz_pos}: size={size}")
        print(f"  Sample size (0=variable): {sample_size}")
        print(f"  Sample count: {sample_count}")

        if sample_size == 0 and sample_count > 0:
            # 개별 샘플 크기들
            sizes = []
            for j in range(min(10, sample_count)):
                s = struct.unpack(">I", stsz_data[20 + j * 4 : 24 + j * 4])[0]
                sizes.append(s)
            print(f"  First 10 sample sizes: {sizes}")

            # 총 데이터 크기 계산
            total_size = sum(
                struct.unpack(">I", stsz_data[20 + j * 4 : 24 + j * 4])[0]
                for j in range(sample_count)
            )
            print(f"  Total data size: {total_size:,} bytes")

    # mdat 박스 찾기
    mdat_pos = data.find(b"mdat")
    if mdat_pos > 0:
        mdat_pos -= 4
        mdat_size = struct.unpack(">I", data[mdat_pos : mdat_pos + 4])[0]
        print(f"\n\n[MDAT] mdat box at offset {mdat_pos} (0x{mdat_pos:x})")
        print(f"  Size: {mdat_size:,} bytes")
        print(f"  Data starts at: {mdat_pos + 8} (0x{mdat_pos + 8:x})")

    print("\n" + "=" * 60)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Analyze MP4 file structure')
    parser.add_argument('input_file', help='Path to the MP4 file to analyze')
    args = parser.parse_args()

    analyze_mp4(args.input_file)
