#!/usr/bin/env python3
"""
MP4 파일의 mvhd 박스를 분석하는 스크립트
timescale, duration, version 등을 확인
Usage: python analyze_mvhd.py file.mp4
"""

import sys
import struct

def analyze_mvhd(filename):
    """mvhd 박스 파싱 및 분석"""
    with open(filename, 'rb') as f:
        # moov 박스 찾기 (일반적으로 offset 32에 위치)
        f.seek(32)
        
        moov_size = int.from_bytes(f.read(4), 'big')
        moov_type = f.read(4).decode('ascii')
        
        if moov_type != 'moov':
            print(f"Error: Expected 'moov' box at offset 32, found '{moov_type}'")
            return
        
        print(f"moov box: size = {moov_size} bytes")
        
        # mvhd 박스 읽기
        mvhd_size = int.from_bytes(f.read(4), 'big')
        mvhd_type = f.read(4).decode('ascii')
        
        if mvhd_type != 'mvhd':
            print(f"Error: Expected 'mvhd' box, found '{mvhd_type}'")
            return
        
        print(f"mvhd box: size = {mvhd_size} bytes")
        
        # mvhd 내용 파싱
        version_flags = f.read(4)
        version = version_flags[0]
        flags = version_flags[1:].hex()
        
        print(f"version = {version}, flags = 0x{flags}")
        
        if version == 1:
            # version 1: 64-bit timestamps
            creation_time = int.from_bytes(f.read(8), 'big')
            modification_time = int.from_bytes(f.read(8), 'big')
            timescale = int.from_bytes(f.read(4), 'big')
            duration = int.from_bytes(f.read(8), 'big')
        else:
            # version 0: 32-bit timestamps
            creation_time = int.from_bytes(f.read(4), 'big')
            modification_time = int.from_bytes(f.read(4), 'big')
            timescale = int.from_bytes(f.read(4), 'big')
            duration = int.from_bytes(f.read(4), 'big')
        
        print(f"creation_time = {creation_time}")
        print(f"modification_time = {modification_time}")
        print(f"timescale = {timescale} (0x{timescale:x})")
        print(f"duration = {duration} (0x{duration:x})")
        
        if timescale > 0:
            seconds = duration / timescale
            print(f"duration in seconds = {seconds:.3f}s")

def main():
    if len(sys.argv) != 2:
        print("Usage: python analyze_mvhd.py file.mp4")
        sys.exit(1)
    
    filename = sys.argv[1]
    print(f"Analyzing: {filename}\n")
    analyze_mvhd(filename)

if __name__ == '__main__':
    main()
