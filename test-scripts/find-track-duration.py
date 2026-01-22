import struct

def parse_full_box_header(data, offset):
    version = data[offset]
    flags = struct.unpack('>I', b'\x00' + data[offset+1:offset+4])[0]
    return version, flags

def find_in_container(data, offset, size, target_type):
    """컨테이너 박스 내에서 특정 타입 찾기"""
    end = offset + size
    current = offset + 8  # 박스 헤더 건너뛰기
    
    results = []
    while current < end:
        if current + 8 > end:
            break
        box_size = struct.unpack('>I', data[current:current+4])[0]
        box_type = data[current+4:current+8]
        
        if box_type == target_type:
            results.append((current, box_size))
        
        if box_size == 0 or current + box_size > end:
            break
        current += box_size
    
    return results

def parse_tkhd(data, offset):
    version, flags = parse_full_box_header(data, offset + 8)
    
    if version == 1:
        duration_offset = offset + 8 + 4 + 8 + 8 + 4 + 4  # header + creation + modification + track_id + reserved
        duration = struct.unpack('>Q', data[duration_offset:duration_offset+8])[0]
    else:
        duration_offset = offset + 8 + 4 + 4 + 4 + 4 + 4  # header + creation + modification + track_id + reserved
        duration = struct.unpack('>I', data[duration_offset:duration_offset+4])[0]
    
    return version, duration

def parse_mdhd(data, offset):
    version, flags = parse_full_box_header(data, offset + 8)
    
    if version == 1:
        timescale_offset = offset + 8 + 4 + 8 + 8
        timescale = struct.unpack('>I', data[timescale_offset:timescale_offset+4])[0]
        duration = struct.unpack('>Q', data[timescale_offset+4:timescale_offset+12])[0]
    else:
        timescale_offset = offset + 8 + 4 + 4 + 4
        timescale = struct.unpack('>I', data[timescale_offset:timescale_offset+4])[0]
        duration = struct.unpack('>I', data[timescale_offset+4:timescale_offset+8])[0]
    
    return version, timescale, duration

for filename in ['input-5mb.mp4', 'fixed-5mb.mp4']:
    print(f"\n{filename}:")
    print("="*60)
    
    with open(filename, 'rb') as f:
        data = f.read()
    
    # moov 찾기
    offset = 0
    while offset < len(data):
        if offset + 8 > len(data):
            break
        size = struct.unpack('>I', data[offset:offset+4])[0]
        box_type = data[offset+4:offset+8]
        
        if box_type == b'moov':
            # trak 찾기
            traks = find_in_container(data, offset, size, b'trak')
            
            for idx, (trak_offset, trak_size) in enumerate(traks):
                print(f"\nTrack {idx + 1}:")
                
                # tkhd 찾기
                tkhds = find_in_container(data, trak_offset, trak_size, b'tkhd')
                if tkhds:
                    tkhd_offset, tkhd_size = tkhds[0]
                    version, duration = parse_tkhd(data, tkhd_offset)
                    print(f"  tkhd: version={version}, duration={duration}")
                
                # mdia 찾기
                mdias = find_in_container(data, trak_offset, trak_size, b'mdia')
                if mdias:
                    mdia_offset, mdia_size = mdias[0]
                    
                    # mdhd 찾기
                    mdhds = find_in_container(data, mdia_offset, mdia_size, b'mdhd')
                    if mdhds:
                        mdhd_offset, mdhd_size = mdhds[0]
                        version, timescale, duration = parse_mdhd(data, mdhd_offset)
                        print(f"  mdhd: version={version}, timescale={timescale}, duration={duration}, seconds={duration/timescale:.2f}")
            
            break
        
        if size == 0 or offset + size > len(data):
            break
        offset += size
