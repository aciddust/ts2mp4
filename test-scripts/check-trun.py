import struct

def parse_full_box_header(data, offset):
    version = data[offset]
    flags = struct.unpack('>I', b'\x00' + data[offset+1:offset+4])[0]
    return version, flags

def find_boxes(data, start, end, box_type):
    results = []
    offset = start
    
    while offset < end:
        if offset + 8 > end:
            break
        size = struct.unpack('>I', data[offset:offset+4])[0]
        btype = data[offset+4:offset+8]
        
        if btype == box_type:
            results.append((offset, size))
        
        if size == 0 or offset + size > end:
            break
        offset += size
    
    return results

def parse_trun(data, offset, size):
    """trun 박스 파싱"""
    version, flags = parse_full_box_header(data, offset + 8)
    
    # flags 비트 확인
    data_offset_present = (flags & 0x000001) != 0
    first_sample_flags_present = (flags & 0x000004) != 0
    sample_duration_present = (flags & 0x000100) != 0
    sample_size_present = (flags & 0x000200) != 0
    sample_flags_present = (flags & 0x000400) != 0
    sample_composition_time_present = (flags & 0x000800) != 0
    
    pos = offset + 12  # version/flags 다음
    
    sample_count = struct.unpack('>I', data[pos:pos+4])[0]
    pos += 4
    
    if data_offset_present:
        pos += 4
    if first_sample_flags_present:
        pos += 4
    
    # 첫 샘플의 duration만 확인
    first_sample_duration = None
    if sample_duration_present and sample_count > 0:
        first_sample_duration = struct.unpack('>I', data[pos:pos+4])[0]
    
    return {
        'sample_count': sample_count,
        'first_sample_duration': first_sample_duration,
        'flags': flags
    }

for filename in ['input-5mb.mp4', 'fixed-5mb.mp4']:
    print(f"\n{filename}:")
    print("="*60)
    
    with open(filename, 'rb') as f:
        data = f.read()
    
    # 처음 3개 moof만 확인
    moofs = find_boxes(data, 0, len(data), b'moof')
    
    for idx, (moof_offset, moof_size) in enumerate(moofs[:3]):
        print(f"\nmoof #{idx + 1}:")
        
        trafs = find_boxes(data, moof_offset + 8, moof_offset + moof_size, b'traf')
        
        for traf_idx, (traf_offset, traf_size) in enumerate(trafs):
            truns = find_boxes(data, traf_offset + 8, traf_offset + traf_size, b'trun')
            
            if truns:
                trun_offset, trun_size = truns[0]
                trun_info = parse_trun(data, trun_offset, trun_size)
                print(f"  traf #{traf_idx + 1}: sample_count={trun_info['sample_count']}, "
                      f"first_sample_duration={trun_info['first_sample_duration']}, "
                      f"flags=0x{trun_info['flags']:06x}")
