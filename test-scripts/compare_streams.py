#!/usr/bin/env python3
"""
두 MP4 파일의 스트림 정보를 비교하는 스크립트
ffprobe를 사용하여 코덱, duration, 해상도 등을 비교
Usage: python compare_streams.py file1.mp4 file2.mp4
"""

import sys
import subprocess
import json

def get_stream_info(filename):
    """ffprobe로 스트림 정보 추출"""
    result = subprocess.run([
        'ffprobe', '-v', 'quiet', '-print_format', 'json',
        '-show_format', '-show_streams', filename
    ], capture_output=True, text=True)
    
    return json.loads(result.stdout)

def print_stream_info(filename, info):
    """스트림 정보 출력"""
    print(f"=== {filename} ===")
    
    if 'format' in info:
        fmt = info['format']
        if 'duration' in fmt:
            print(f"Format duration: {float(fmt['duration']):.3f}s")
    
    if 'streams' in info:
        for stream in info['streams']:
            idx = stream.get('index', '?')
            codec_type = stream.get('codec_type', 'unknown')
            codec_name = stream.get('codec_name', 'unknown')
            
            print(f"\nTrack {idx}: {codec_type} - {codec_name}")
            
            if codec_type == 'video':
                width = stream.get('width', '?')
                height = stream.get('height', '?')
                fps = stream.get('r_frame_rate', '?')
                print(f"  Resolution: {width}x{height}")
                print(f"  Frame rate: {fps}")
            elif codec_type == 'audio':
                sample_rate = stream.get('sample_rate', '?')
                channels = stream.get('channels', '?')
                print(f"  Sample rate: {sample_rate}Hz")
                print(f"  Channels: {channels}")
            
            if 'duration' in stream:
                print(f"  Duration: {float(stream['duration']):.3f}s")
            
            if 'tags' in stream:
                handler = stream['tags'].get('handler_name', '')
                if handler:
                    print(f"  Handler: {handler}")

def main():
    if len(sys.argv) != 3:
        print("Usage: python compare_streams.py file1.mp4 file2.mp4")
        sys.exit(1)
    
    file1 = sys.argv[1]
    file2 = sys.argv[2]
    
    info1 = get_stream_info(file1)
    info2 = get_stream_info(file2)
    
    print_stream_info(file1, info1)
    print("\n" + "="*50 + "\n")
    print_stream_info(file2, info2)

if __name__ == '__main__':
    main()
