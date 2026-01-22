import struct

def parse_full_box_header(data, offset):
    version = data[offset]
    flags = struct.unpack('>I', b'\x00' + data[offset+1:offset+4])[0]
    return version, flags

def find_boxes(data, start, end, box_type):
    """범위 내에서 특정 박스 찾기"""
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

def parse_tfdt(data, offset):
    """tfdt 박스에서 baseMediaDecodeTime 읽기"""
    version, flags = parse_full_box_header(data, offset + 8)
    
    if version == 1:
        base_time = struct.unpack('>Q', data[offset+12:offset+20])[0]
    else:
        base_time = struct.unpack('>I', data[offset+12:offset+16])[0]
    
    return version, base_time

for filename in ['input-5mb.mp4', 'fixed-5mb.mp4']:
    print(f"\n{filename}:")
    print("="*60)
    
    with open(filename, 'rb') as f:
        data = f.read()
    
    # 모든 moof 찾기
    moofs = find_boxes(data, 0, len(data), b'moof')
    
    for idx, (moof_offset, moof_size) in enumerate(moofs[:5]):  # 처음 5개만
        print(f"\nmoof #{idx + 1} at offset {moof_offset}:")
        
        # moof 내부의 traf 찾기
        trafs = find_boxes(data, moof_offset + 8, moof_offset + moof_size, b'traf')
        
        for traf_idx, (traf_offset, traf_size) in enumerate(trafs):
            # tfdt 찾기
            tfdts = find_boxes(data, traf_offset + 8, traf_offset + traf_size, b'tfdt')
            
            if tfdts:
                tfdt_offset, tfdt_size = tfdts[0]
                version, base_time = parse_tfdt(data, tfdt_offset)
                
                # 타임스케일은 mdhd에서 가져와야 하지만, 일단 raw 값 표시
                print(f"  traf #{traf_idx + 1}: tfdt version={version}, baseMediaDecodeTime={base_time}")
