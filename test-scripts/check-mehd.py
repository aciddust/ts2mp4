import struct

def find_in_container(data, offset, size, target_type, recursive=False):
    """컨테이너 내부에서 박스 찾기"""
    results = []
    current = offset + 8
    end = offset + size
    
    while current < end:
        if current + 8 > end:
            break
        box_size = struct.unpack('>I', data[current:current+4])[0]
        box_type = data[current+4:current+8]
        
        if box_type == target_type:
            results.append((current, box_size))
        
        # mvex, trak 등 컨테이너 내부도 재귀 검색
        if recursive and box_type in [b'mvex', b'trak', b'mdia']:
            results.extend(find_in_container(data, current, box_size, target_type, True))
        
        if box_size == 0 or current + box_size > end:
            break
        current += box_size
    
    return results

def parse_mehd(data, offset):
    """mehd 박스 파싱"""
    version = data[offset+8]
    
    if version == 1:
        duration = struct.unpack('>Q', data[offset+12:offset+20])[0]
    else:
        duration = struct.unpack('>I', data[offset+12:offset+16])[0]
    
    return version, duration

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
            # mvex (Movie Extends) 찾기
            mvexs = find_in_container(data, offset, size, b'mvex', False)
            
            if mvexs:
                print("  Found mvex box")
                mvex_offset, mvex_size = mvexs[0]
                
                # mehd (Movie Extends Header) 찾기
                mehds = find_in_container(data, mvex_offset, mvex_size, b'mehd', False)
                
                if mehds:
                    mehd_offset, mehd_size = mehds[0]
                    version, duration = parse_mehd(data, mehd_offset)
                    print(f"    mehd: version={version}, fragment_duration={duration}")
                else:
                    print("    No mehd found (duration unspecified)")
            else:
                print("  No mvex box (not a fragmented MP4)")
            
            break
        
        if size == 0 or offset + size > len(data):
            break
        offset += size
