#!/usr/bin/env python3
import struct

def find_boxes_recursive(data, box_type, path=''):
    results = []
    pos = 0
    while pos < len(data):
        if pos + 8 > len(data):
            break
        size = struct.unpack('>I', data[pos:pos+4])[0]
        btype = data[pos+4:pos+8]
        
        current_path = path + '/' + btype.decode('latin1')
        
        if btype == box_type:
            results.append(current_path)
        
        # Container boxes
        if btype in [b'moov', b'trak', b'mdia', b'minf', b'stbl', b'edts']:
            results.extend(find_boxes_recursive(data[pos+8:pos+size], box_type, current_path))
        
        if size == 0:
            break
        pos += size
    return results

with open('answer-5mb.mp4', 'rb') as f:
    data = f.read()

edts_boxes = find_boxes_recursive(data, b'edts')
print('answer-5mb.mp4 edts boxes:', edts_boxes)

with open('defrag-5mb.mp4', 'rb') as f:
    data = f.read()

edts_boxes = find_boxes_recursive(data, b'edts')
print('defrag-5mb.mp4 edts boxes:', edts_boxes)
