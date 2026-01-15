use crate::ts_parser::{parse_ts_packets, MediaData};
use std::io::{self, ErrorKind};

/// Extracts thumbnail data from TS file
/// Returns the first I-frame (keyframe) as raw H.264 NAL units
pub fn extract_thumbnail_from_ts(ts_data: &[u8]) -> io::Result<Vec<u8>> {
    let media_data = parse_ts_packets(ts_data)?;
    extract_first_iframe(&media_data)
}

/// Extracts thumbnail data from MP4 file
/// Returns the first keyframe as raw H.264 data
pub fn extract_thumbnail_from_mp4(mp4_data: &[u8]) -> io::Result<Vec<u8>> {
    // Parse MP4 structure to find the first video sample
    let first_sample = find_first_video_sample(mp4_data)?;
    Ok(first_sample)
}

/// Extracts the first I-frame from parsed media data
fn extract_first_iframe(media_data: &MediaData) -> io::Result<Vec<u8>> {
    if media_data.video_stream.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "No video data found",
        ));
    }

    // Split video stream into frames and find the first I-frame
    let mut offset = 0;
    let data = &media_data.video_stream;

    while offset < data.len() {
        // Find NAL unit start code (0x00 0x00 0x00 0x01 or 0x00 0x00 0x01)
        let start_code_len = if offset + 3 < data.len()
            && data[offset] == 0x00
            && data[offset + 1] == 0x00
            && data[offset + 2] == 0x01
        {
            3
        } else if offset + 4 < data.len()
            && data[offset] == 0x00
            && data[offset + 1] == 0x00
            && data[offset + 2] == 0x00
            && data[offset + 3] == 0x01
        {
            4
        } else {
            offset += 1;
            continue;
        };

        let nal_start = offset + start_code_len;
        if nal_start >= data.len() {
            break;
        }

        let nal_type = data[nal_start] & 0x1F;

        // NAL type 5 is IDR (I-frame), type 1 can also be I-frame
        // We'll look for IDR frame (type 5) for thumbnail
        if nal_type == 5 {
            // Find the end of this frame (next start code or end of data)
            let mut frame_end = nal_start + 1;
            let mut frame_data = Vec::new();

            // Include SPS and PPS if available
            if let Some(ref sps) = media_data.sps {
                frame_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
                frame_data.extend_from_slice(sps);
            }
            if let Some(ref pps) = media_data.pps {
                frame_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
                frame_data.extend_from_slice(pps);
            }

            // Find the complete I-frame
            while frame_end < data.len() {
                // Check for next start code
                if frame_end + 3 <= data.len()
                    && data[frame_end] == 0x00
                    && data[frame_end + 1] == 0x00
                {
                    if data[frame_end + 2] == 0x01 {
                        break;
                    } else if frame_end + 4 <= data.len()
                        && data[frame_end + 2] == 0x00
                        && data[frame_end + 3] == 0x01
                    {
                        break;
                    }
                }
                frame_end += 1;
            }

            // Add the I-frame data
            frame_data.extend_from_slice(&data[offset..frame_end]);

            return Ok(frame_data);
        }

        offset += 1;
    }

    Err(io::Error::new(
        ErrorKind::NotFound,
        "No I-frame found in video stream",
    ))
}

/// Finds the first video sample in an MP4 file
fn find_first_video_sample(mp4_data: &[u8]) -> io::Result<Vec<u8>> {
    // Find mdat box
    let mdat_offset = find_box(mp4_data, b"mdat")?;
    let _mdat_size = read_u32(mp4_data, mdat_offset) as usize;
    let mdat_data_start = mdat_offset + 8;

    if mdat_data_start >= mp4_data.len() {
        return Err(io::Error::new(ErrorKind::InvalidData, "Invalid mdat box"));
    }

    // Find moov box to get sample information
    let moov_offset = find_box(mp4_data, b"moov")?;

    // Find trak box for video
    let trak_offset = find_box_in_container(mp4_data, moov_offset, b"trak")?;

    // Find stbl (sample table) box
    let mdia_offset = find_box_in_container(mp4_data, trak_offset, b"mdia")?;
    let minf_offset = find_box_in_container(mp4_data, mdia_offset, b"minf")?;
    let stbl_offset = find_box_in_container(mp4_data, minf_offset, b"stbl")?;

    // Find stsz (sample sizes) to get first sample size
    let stsz_offset = find_box_in_container(mp4_data, stbl_offset, b"stsz")?;
    let first_sample_size = read_first_sample_size(mp4_data, stsz_offset)?;

    // Find stss (sync samples) to verify it's a keyframe
    // For thumbnail, we assume the first sample is a keyframe

    // Extract first sample from mdat
    if mdat_data_start + first_sample_size > mp4_data.len() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "Sample size exceeds mdat bounds",
        ));
    }

    let sample_data = mp4_data[mdat_data_start..mdat_data_start + first_sample_size].to_vec();

    // Convert from AVCC format to Annex B format (add start codes)
    let annexb_data = convert_avcc_to_annexb(&sample_data)?;

    Ok(annexb_data)
}

/// Converts AVCC format (length-prefixed NAL units) to Annex B format (start code prefixed)
fn convert_avcc_to_annexb(avcc_data: &[u8]) -> io::Result<Vec<u8>> {
    let mut annexb = Vec::new();
    let mut offset = 0;

    while offset + 4 <= avcc_data.len() {
        // Read NAL unit length (4 bytes)
        let nal_length = read_u32(avcc_data, offset) as usize;
        offset += 4;

        if offset + nal_length > avcc_data.len() {
            break;
        }

        // Add start code
        annexb.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
        // Add NAL unit
        annexb.extend_from_slice(&avcc_data[offset..offset + nal_length]);

        offset += nal_length;
    }

    Ok(annexb)
}

/// Finds a box in MP4 data
fn find_box(data: &[u8], box_type: &[u8; 4]) -> io::Result<usize> {
    let mut offset = 0;

    while offset + 8 <= data.len() {
        let size = read_u32(data, offset) as usize;
        if size < 8 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid box size"));
        }

        if &data[offset + 4..offset + 8] == box_type {
            return Ok(offset);
        }

        offset += size;
    }

    Err(io::Error::new(
        ErrorKind::NotFound,
        format!(
            "Box {:?} not found",
            std::str::from_utf8(box_type).unwrap_or("unknown")
        ),
    ))
}

/// Finds a box within a container box
fn find_box_in_container(
    data: &[u8],
    container_offset: usize,
    box_type: &[u8; 4],
) -> io::Result<usize> {
    let container_size = read_u32(data, container_offset) as usize;
    let mut offset = container_offset + 8;
    let container_end = container_offset + container_size;

    while offset + 8 <= container_end && offset + 8 <= data.len() {
        let size = read_u32(data, offset) as usize;
        if size < 8 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid box size"));
        }

        if &data[offset + 4..offset + 8] == box_type {
            return Ok(offset);
        }

        offset += size;
    }

    Err(io::Error::new(
        ErrorKind::NotFound,
        format!(
            "Box {:?} not found in container",
            std::str::from_utf8(box_type).unwrap_or("unknown")
        ),
    ))
}

/// Reads first sample size from stsz box
fn read_first_sample_size(data: &[u8], stsz_offset: usize) -> io::Result<usize> {
    // stsz box structure:
    // 4 bytes: size
    // 4 bytes: type ('stsz')
    // 1 byte: version
    // 3 bytes: flags
    // 4 bytes: sample_size (if 0, then each sample has different size)
    // 4 bytes: sample_count
    // [4 bytes * sample_count]: sample sizes (if sample_size == 0)

    if stsz_offset + 20 > data.len() {
        return Err(io::Error::new(ErrorKind::InvalidData, "Invalid stsz box"));
    }

    let sample_size = read_u32(data, stsz_offset + 12) as usize;

    if sample_size != 0 {
        // All samples have the same size
        return Ok(sample_size);
    }

    // Read first sample size from the table
    if stsz_offset + 24 > data.len() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "Invalid stsz box - no sample sizes",
        ));
    }

    let first_sample_size = read_u32(data, stsz_offset + 20) as usize;
    Ok(first_sample_size)
}

/// Reads a big-endian u32 from data
fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_avcc_to_annexb() {
        // Simple test with one NAL unit
        let avcc = vec![
            0x00, 0x00, 0x00, 0x05, // length = 5
            0x67, 0x01, 0x02, 0x03, 0x04, // NAL data
        ];

        let annexb = convert_avcc_to_annexb(&avcc).unwrap();

        assert_eq!(annexb[0..4], [0x00, 0x00, 0x00, 0x01]); // start code
        assert_eq!(annexb[4..9], [0x67, 0x01, 0x02, 0x03, 0x04]); // NAL data
    }
}
