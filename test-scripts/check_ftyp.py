#!/usr/bin/env python3
"""
MP4 파일의 ftyp 박스를 확인하는 스크립트
major_brand, minor_version, compatible_brands 출력
Usage: python check_ftyp.py file.mp4
"""

import sys

def check_ftyp(filename):
    """ftyp 박스 파싱"""
    with open(filename, 'rb') as f:
        # ftyp은 파일 시작 부분에 위치
        size = int.from_bytes(f.read(4), 'big')
        box_type = f.read(4).decode('ascii')
        
        if box_type != 'ftyp':
            print(f"Error: Expected 'ftyp' box at start, found '{box_type}'")
            return
        
        print(f"ftyp box: size = {size} bytes")
        
        # major_brand (4 bytes)
        major_brand = f.read(4).decode('ascii')
        print(f"major_brand = '{major_brand}'")
        
        # minor_version (4 bytes)
        minor_version = int.from_bytes(f.read(4), 'big')
        print(f"minor_version = {minor_version}")
        
        # compatible_brands (나머지)
        remaining_bytes = size - 16  # 8 (size+type) + 4 (major_brand) + 4 (minor_version)
        compatible_brands = []
        
        while remaining_bytes >= 4:
            brand = f.read(4).decode('ascii', errors='ignore')
            compatible_brands.append(brand)
            remaining_bytes -= 4
        
        print(f"compatible_brands = {compatible_brands}")
        
        # QuickTime 호환성 체크
        if major_brand == 'isom':
            print("✓ QuickTime compatible (isom)")
        elif major_brand == 'iso6':
            print("⚠ Fragmented MP4 (iso6) - may not work in QuickTime")
        else:
            print(f"? Unknown brand: {major_brand}")

def main():
    if len(sys.argv) != 2:
        print("Usage: python check_ftyp.py file.mp4")
        sys.exit(1)
    
    filename = sys.argv[1]
    print(f"Checking: {filename}\n")
    check_ftyp(filename)

if __name__ == '__main__':
    main()
