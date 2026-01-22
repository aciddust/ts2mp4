#!/usr/bin/env python3
import struct
import sys

if len(sys.argv) < 2:
    print("Usage: python check_data_offset.py <mp4_file>")
    sys.exit(1)

filename = sys.argv[1]

with open(filename, 'rb') as f:
    data = f.read()

# 첫 번째 moof 찾기
pos = 0
moof_count = 0
while pos < len(data) and moof_count < 2:  # 처음 2개만
    if pos + 8 > len(data):
        break
    size = struct.unpack('>I', data[pos:pos+4])[0]
    box_type = data[pos+4:pos+8]
    
    if box_type == b'moof':
        print(f'\n========== moof #{moof_count+1} at offset {pos}, size={size} ==========')
        moof_start = pos
        
        # moof 내부의 traf 찾기
        moof_data = data[pos+8:pos+size]
        traf_pos = 0
        traf_idx = 0
        while traf_pos < len(moof_data):
            if traf_pos + 8 > len(moof_data):
                break
            traf_size = struct.unpack('>I', moof_data[traf_pos:traf_pos+4])[0]
            traf_type = moof_data[traf_pos+4:traf_pos+8]
            
            if traf_type == b'traf':
                print(f'\n  traf #{traf_idx} at offset {traf_pos} (absolute: {pos+8+traf_pos}), size={traf_size}')
                
                # tfhd에서 track_id 추출
                traf_data = moof_data[traf_pos+8:traf_pos+traf_size]
                sub_pos = 0
                while sub_pos < len(traf_data):
                    if sub_pos + 8 > len(traf_data):
                        break
                    sub_size = struct.unpack('>I', traf_data[sub_pos:sub_pos+4])[0]
                    sub_type = traf_data[sub_pos+4:sub_pos+8]
                    
                    if sub_type == b'tfhd' and len(traf_data) >= sub_pos + 12:
                        track_id = struct.unpack('>I', traf_data[sub_pos+12:sub_pos+16])[0]
                        print(f'    track_id={track_id}')
                    
                    if sub_type == b'trun':
                        # trun 파싱
                        trun_data = traf_data[sub_pos+8:sub_pos+sub_size]
                        if len(trun_data) >= 8:
                            flags = struct.unpack('>I', b'\x00' + trun_data[1:4])[0]
                            sample_count = struct.unpack('>I', trun_data[4:8])[0]
                            print(f'    trun: flags={hex(flags)}, samples={sample_count}')
                            
                            # data_offset (flag 0x000001)
                            if flags & 0x000001 and len(trun_data) >= 12:
                                data_offset = struct.unpack('>i', trun_data[8:12])[0]
                                print(f'      data_offset={data_offset} (from moof start)')
                                print(f'      absolute offset={moof_start + data_offset}')
                    
                    sub_pos += sub_size
                
                traf_idx += 1
            
            traf_pos += traf_size
        
        moof_count += 1
    
    pos += size

print()
