#!/usr/bin/env python3
"""
MP4 파일의 프레임 디코딩을 검증하는 스크립트
ffmpeg로 모든 프레임이 정상 디코딩되는지 확인
Usage: python verify_frames.py file.mp4
"""

import sys
import subprocess
import re

def verify_frames(filename):
    """ffmpeg로 프레임 디코딩 검증"""
    print(f"Verifying frames in: {filename}")
    print("Decoding all frames with ffmpeg...\n")
    
    result = subprocess.run([
        'ffmpeg', '-v', 'error', '-i', filename, '-f', 'null', '-'
    ], capture_output=True, text=True)
    
    if result.returncode != 0:
        print("❌ Error during decoding:")
        print(result.stderr)
        return False
    
    if result.stderr:
        print("⚠ Warnings:")
        print(result.stderr)
    
    # 통계 정보 가져오기
    result = subprocess.run([
        'ffmpeg', '-i', filename, '-f', 'null', '-'
    ], capture_output=True, text=True, stderr=subprocess.STDOUT)
    
    # frame 수 추출
    frame_match = re.search(r'frame=\s*(\d+)', result.stdout)
    if frame_match:
        frame_count = int(frame_match.group(1))
        print(f"✓ Successfully decoded {frame_count} frames")
    else:
        print("✓ All frames decoded successfully")
    
    # duration 추출
    time_match = re.search(r'time=(\d+:\d+:\d+\.\d+)', result.stdout)
    if time_match:
        duration = time_match.group(1)
        print(f"✓ Duration: {duration}")
    
    return True

def main():
    if len(sys.argv) != 2:
        print("Usage: python verify_frames.py file.mp4")
        sys.exit(1)
    
    filename = sys.argv[1]
    
    if verify_frames(filename):
        print("\n✅ File is valid and all frames can be decoded")
        sys.exit(0)
    else:
        print("\n❌ File has errors")
        sys.exit(1)

if __name__ == '__main__':
    main()
