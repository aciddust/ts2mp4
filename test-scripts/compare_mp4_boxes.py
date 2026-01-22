#!/usr/bin/env python3
"""
두 MP4 파일의 박스 구조를 비교하는 스크립트
Usage: python compare_mp4_boxes.py file1.mp4 file2.mp4
"""

import sys
import subprocess

def get_box_structure(filename):
    """xxd로 파일의 박스 구조 확인"""
    result = subprocess.run(['xxd', '-l', '200', '-s', '32', filename], 
                          capture_output=True, text=True)
    return result.stdout

def main():
    if len(sys.argv) != 3:
        print("Usage: python compare_mp4_boxes.py file1.mp4 file2.mp4")
        sys.exit(1)
    
    file1 = sys.argv[1]
    file2 = sys.argv[2]
    
    print(f"=== {file1} ===")
    print(get_box_structure(file1))
    
    print(f"\n=== {file2} ===")
    print(get_box_structure(file2))

if __name__ == '__main__':
    main()
