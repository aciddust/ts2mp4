#!/usr/bin/env python3
"""
Fragmented MP4와 일반 MP4 변환 테스트 스크립트
입력 파일을 defragment하고 QuickTime 호환성 검증
Usage: python test_defragment.py input.mp4 output.mp4
"""

import sys
import subprocess
import os

def run_command(cmd, description):
    """명령어 실행 및 결과 출력"""
    print(f"\n{'='*60}")
    print(f"{description}")
    print(f"{'='*60}")
    print(f"Command: {' '.join(cmd)}\n")
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.stdout:
        print(result.stdout)
    if result.stderr:
        print(result.stderr)
    
    return result.returncode == 0

def main():
    if len(sys.argv) != 3:
        print("Usage: python test_defragment.py input.mp4 output.mp4")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    
    if not os.path.exists(input_file):
        print(f"Error: Input file '{input_file}' not found")
        sys.exit(1)
    
    # 1. 원본 파일 분석
    run_command(
        ['python3', 'test-scripts/check_ftyp.py', input_file],
        "Step 1: Check input file ftyp"
    )
    
    run_command(
        ['python3', 'test-scripts/analyze_mvhd.py', input_file],
        "Step 2: Analyze input file mvhd"
    )
    
    # 2. Defragment 실행
    success = run_command(
        ['./target/release/ts2mp4', 'convert', '--input', input_file, '--output', output_file, '-r'],
        "Step 3: Defragment MP4"
    )
    
    if not success:
        print("\n❌ Defragmentation failed")
        sys.exit(1)
    
    # 3. 출력 파일 분석
    run_command(
        ['python3', 'test-scripts/check_ftyp.py', output_file],
        "Step 4: Check output file ftyp"
    )
    
    run_command(
        ['python3', 'test-scripts/analyze_mvhd.py', output_file],
        "Step 5: Analyze output file mvhd"
    )
    
    # 4. 스트림 비교
    run_command(
        ['python3', 'test-scripts/compare_streams.py', input_file, output_file],
        "Step 6: Compare streams"
    )
    
    # 5. 프레임 검증
    run_command(
        ['python3', 'test-scripts/verify_frames.py', output_file],
        "Step 7: Verify all frames"
    )
    
    print(f"\n{'='*60}")
    print("✅ Defragmentation test completed successfully!")
    print(f"Output file: {output_file}")
    print("You can now test it in QuickTime Player")
    print(f"{'='*60}")

if __name__ == '__main__':
    main()
