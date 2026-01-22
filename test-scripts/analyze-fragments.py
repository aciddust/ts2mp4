import struct

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

def parse_tfdt(data, offset):
    version = data[offset + 8]
    
    if version == 1:
        base_time = struct.unpack('>Q', data[offset+12:offset+20])[0]
    else:
        base_time = struct.unpack('>I', data[offset+12:offset+16])[0]
    
    return version, base_time

def parse_trun(data, offset, size):
    version = data[offset + 8]
    flags = struct.unpack('>I', b'\x00' + data[offset+9:offset+12])[0]
    
    sample_duration_present = (flags & 0x000100) != 0
    
    pos = offset + 12
    sample_count = struct.unpack('>I', data[pos:pos+4])[0]
    
    return sample_count, sample_duration_present

# input-5mb.mp4 분석
with open('input-5mb.mp4', 'rb') as f:
    data = f.read()

moofs = find_boxes(data, 0, len(data), b'moof')

print(f"Total moofs: {len(moofs)}\n")

for idx, (moof_offset, moof_size) in enumerate(moofs):
    print(f"moof #{idx + 1}:")
    
    trafs = find_boxes(data, moof_offset + 8, moof_offset + moof_size, b'traf')
    
    for traf_idx, (traf_offset, traf_size) in enumerate(trafs):
        tfdts = find_boxes(data, traf_offset + 8, traf_offset + traf_size, b'tfdt')
        truns = find_boxes(data, traf_offset + 8, traf_offset + traf_size, b'trun')
        
        tfdt_time = 0
        if tfdts:
            tfdt_offset, _ = tfdts[0]
            _, tfdt_time = parse_tfdt(data, tfdt_offset)
        
        sample_count = 0
        has_duration = False
        if truns:
            trun_offset, trun_size = truns[0]
            sample_count, has_duration = parse_trun(data, trun_offset, trun_size)
        
        print(f"  traf #{traf_idx + 1}: tfdt={tfdt_time}, samples={sample_count}, has_duration={has_duration}")
