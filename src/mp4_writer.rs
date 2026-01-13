use crate::ts_parser::MediaData;
use std::io::{self, ErrorKind};

pub fn create_mp4(media_data: MediaData) -> io::Result<Vec<u8>> {
    if media_data.video_stream.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "No video data found",
        ));
    }

    let mut mp4_buffer = Vec::new();
    let mut mdat_data = Vec::new();
    let mut sample_sizes = Vec::new();

    // Split video stream into frames (access units)
    let frames = split_into_frames(&media_data.video_stream);

    // Convert each frame from Annex B to AVCC format
    for frame in frames {
        let before_size = mdat_data.len();
        convert_annexb_to_avcc(&frame, &mut mdat_data);
        let after_size = mdat_data.len();
        let sample_size = after_size - before_size;

        // Only add sample if it has actual data
        if sample_size > 0 {
            sample_sizes.push(sample_size as u32);
        }
    }

    let mdat_size = mdat_data.len();

    // Calculate composition time offsets (for B-frames)
    let composition_offsets = calculate_composition_offsets(&media_data.frame_timestamps);

    // Write ftyp box
    write_ftyp(&mut mp4_buffer);

    // Write moov box (metadata) - must come before mdat for fast start
    write_moov(
        &mut mp4_buffer,
        &media_data,
        mdat_size,
        &sample_sizes,
        &composition_offsets,
    )?;

    // Write mdat box
    write_box_header(&mut mp4_buffer, "mdat", (8 + mdat_data.len()) as u32);
    mp4_buffer.extend_from_slice(&mdat_data);

    Ok(mp4_buffer)
}

fn calculate_composition_offsets(timestamps: &[(Option<u64>, Option<u64>)]) -> Vec<i32> {
    if timestamps.is_empty() {
        return Vec::new();
    }

    // Find the minimum DTS to normalize decode timestamps
    let min_dts = timestamps
        .iter()
        .filter_map(|(_, dts)| *dts)
        .min()
        .unwrap_or(0);

    let mut offsets = Vec::new();

    for (pts, dts) in timestamps {
        let offset = match (pts, dts) {
            (Some(p), Some(d)) => {
                // Normalize both PTS and DTS by subtracting min_dts
                let normalized_pts = (*p as i64 - min_dts as i64) as i32;
                let normalized_dts = (*d as i64 - min_dts as i64) as i32;
                normalized_pts - normalized_dts
            }
            _ => 0,
        };
        offsets.push(offset);
    }

    // Adjust all offsets so that the minimum presentation time (DTS + ctts) is 0
    // This ensures the video starts at time 0 without negative DTS
    if let Some(&first_offset) = offsets.first() {
        let adjustment = first_offset;
        for offset in &mut offsets {
            *offset -= adjustment;
        }
    }

    offsets
}

fn split_into_frames(video_stream: &[u8]) -> Vec<Vec<u8>> {
    let mut frames = Vec::new();
    let mut current_frame = Vec::new();
    let mut i = 0;

    while i < video_stream.len() {
        // Find start code
        let start_code_len = if i + 3 < video_stream.len()
            && video_stream[i] == 0x00
            && video_stream[i + 1] == 0x00
            && video_stream[i + 2] == 0x00
            && video_stream[i + 3] == 0x01
        {
            4
        } else if i + 2 < video_stream.len()
            && video_stream[i] == 0x00
            && video_stream[i + 1] == 0x00
            && video_stream[i + 2] == 0x01
        {
            3
        } else {
            i += 1;
            continue;
        };

        let nal_start = i + start_code_len;
        if nal_start >= video_stream.len() {
            break;
        }

        let nal_type = video_stream[nal_start] & 0x1F;

        // Check if this is the start of a new frame
        // Only AUD (NAL type 9) marks a true frame boundary
        // NAL types 1 and 5 are slices that can appear multiple times in one frame
        let is_frame_start = nal_type == 9;

        // If we found a frame start and we have data in current_frame, save it
        if is_frame_start && !current_frame.is_empty() {
            frames.push(current_frame.clone());
            current_frame.clear();
        }

        // Find end of this NAL unit
        let mut nal_end = nal_start + 1;
        while nal_end + 2 < video_stream.len() {
            if video_stream[nal_end] == 0x00 && video_stream[nal_end + 1] == 0x00 {
                if nal_end + 2 < video_stream.len() && video_stream[nal_end + 2] == 0x01 {
                    break;
                } else if nal_end + 3 < video_stream.len()
                    && video_stream[nal_end + 2] == 0x00
                    && video_stream[nal_end + 3] == 0x01
                {
                    break;
                }
            }
            nal_end += 1;
        }

        if nal_end > video_stream.len() {
            nal_end = video_stream.len();
        }

        // Add this NAL unit to current frame
        current_frame.extend_from_slice(&video_stream[i..nal_end]);

        i = nal_end;
    }

    // Don't forget the last frame
    if !current_frame.is_empty() {
        frames.push(current_frame);
    }

    frames
}

fn extract_aac_frames(pes_payload: &[u8]) -> Vec<Vec<u8>> {
    let mut frames = Vec::new();
    let mut offset = 0;

    while offset + 7 < pes_payload.len() {
        // Check for ADTS sync word (0xFFF)
        if pes_payload[offset] != 0xFF || (pes_payload[offset + 1] & 0xF0) != 0xF0 {
            offset += 1;
            continue;
        }

        // Parse ADTS header
        let protection_absent = (pes_payload[offset + 1] & 0x01) == 1;
        let frame_length = (((pes_payload[offset + 3] as usize & 0x03) << 11)
            | ((pes_payload[offset + 4] as usize) << 3)
            | ((pes_payload[offset + 5] as usize) >> 5)) as usize;

        if frame_length < 7 || offset + frame_length > pes_payload.len() {
            break;
        }

        // Calculate ADTS header size
        let header_size = if protection_absent { 7 } else { 9 };

        // Extract raw AAC frame (without ADTS header)
        if offset + header_size < offset + frame_length {
            let aac_data = &pes_payload[offset + header_size..offset + frame_length];
            frames.push(aac_data.to_vec());
        }

        offset += frame_length;
    }

    frames
}

fn convert_annexb_to_avcc(data: &[u8], output: &mut Vec<u8>) {
    let mut i = 0;

    while i < data.len() {
        // Find start code (0x00 0x00 0x01 or 0x00 0x00 0x00 0x01)
        let start_code_len = if i + 3 < data.len()
            && data[i] == 0x00
            && data[i + 1] == 0x00
            && data[i + 2] == 0x00
            && data[i + 3] == 0x01
        {
            4
        } else if i + 2 < data.len()
            && data[i] == 0x00
            && data[i + 1] == 0x00
            && data[i + 2] == 0x01
        {
            3
        } else {
            i += 1;
            continue;
        };

        let nal_start = i + start_code_len;
        if nal_start >= data.len() {
            break;
        }

        // Get NAL type
        let nal_type = data[nal_start] & 0x1F;

        // Find next start code
        let mut nal_end = nal_start + 1;
        let mut found_end = false;

        while nal_end + 2 < data.len() {
            if data[nal_end] == 0x00 && data[nal_end + 1] == 0x00 {
                if nal_end + 2 < data.len() && data[nal_end + 2] == 0x01 {
                    found_end = true;
                    break;
                } else if nal_end + 3 < data.len()
                    && data[nal_end + 2] == 0x00
                    && data[nal_end + 3] == 0x01
                {
                    found_end = true;
                    break;
                }
            }
            nal_end += 1;
        }

        if !found_end {
            nal_end = data.len();
        }

        // Skip SPS (7), PPS (8), and AUD (9) - these go in avcC or are not needed in mdat
        // Only include actual video frame data (slice types: 1, 5, etc.)
        if nal_type != 7 && nal_type != 8 && nal_type != 9 {
            // Write NAL unit with length prefix (4-byte big-endian)
            let nal_size = nal_end - nal_start;
            if nal_size > 0 {
                output.extend_from_slice(&(nal_size as u32).to_be_bytes());
                output.extend_from_slice(&data[nal_start..nal_end]);
            } else {
                println!(
                    "WARNING: Zero-size NAL unit type {} at position {}",
                    nal_type, i
                );
            }
        }

        i = nal_end;
    }
}

fn write_box_header(buffer: &mut Vec<u8>, box_type: &str, size: u32) {
    buffer.extend_from_slice(&size.to_be_bytes());
    buffer.extend_from_slice(box_type.as_bytes());
}

fn write_ftyp(buffer: &mut Vec<u8>) {
    let ftyp_data = [
        // Box size (28 bytes)
        0x00, 0x00, 0x00, 0x1C, // Box type 'ftyp'
        b'f', b't', b'y', b'p', // Major brand 'isom'
        b'i', b's', b'o', b'm', // Minor version
        0x00, 0x00, 0x02, 0x00, // Compatible brands
        b'i', b's', b'o', b'm', b'i', b's', b'o', b'2', b'm', b'p', b'4', b'1',
    ];
    buffer.extend_from_slice(&ftyp_data);
}

fn write_moov(
    buffer: &mut Vec<u8>,
    media_data: &MediaData,
    mdat_size: usize,
    sample_sizes: &[u32],
    composition_offsets: &[i32],
) -> io::Result<()> {
    let mut moov_data = Vec::new();

    // Write mvhd (movie header)
    write_mvhd(&mut moov_data, media_data, sample_sizes.len());

    // Write trak (track) for video
    write_video_trak(
        &mut moov_data,
        media_data,
        mdat_size,
        sample_sizes,
        composition_offsets,
    )?;

    // Calculate exact moov size (8 for box header + data)
    let moov_size = 8 + moov_data.len();

    // Write moov box header
    write_box_header(buffer, "moov", moov_size as u32);
    buffer.extend_from_slice(&moov_data);

    Ok(())
}

fn write_mvhd(buffer: &mut Vec<u8>, media_data: &MediaData, sample_count: usize) {
    let timescale = 90000; // Common for video
    let duration = sample_count as u32 * 3000; // Approximate

    let mvhd_data = vec![
        0x00,
        0x00,
        0x00,
        0x6C, // Box size (108 bytes)
        b'm',
        b'v',
        b'h',
        b'd', // Box type
        0x00, // Version
        0x00,
        0x00,
        0x00, // Flags
        0x00,
        0x00,
        0x00,
        0x00, // Creation time
        0x00,
        0x00,
        0x00,
        0x00, // Modification time
        (timescale >> 24) as u8,
        (timescale >> 16) as u8,
        (timescale >> 8) as u8,
        timescale as u8,
        (duration >> 24) as u8,
        (duration >> 16) as u8,
        (duration >> 8) as u8,
        duration as u8,
        0x00,
        0x01,
        0x00,
        0x00, // Rate (1.0)
        0x01,
        0x00, // Volume (1.0)
        0x00,
        0x00, // Reserved
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Reserved
        0x00,
        0x01,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x01,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x40,
        0x00,
        0x00,
        0x00, // Matrix
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Pre-defined
        0x00,
        0x00,
        0x00,
        0x02, // Next track ID
    ];

    buffer.extend_from_slice(&mvhd_data);
}

fn write_video_trak(
    buffer: &mut Vec<u8>,
    media_data: &MediaData,
    mdat_size: usize,
    sample_sizes: &[u32],
    composition_offsets: &[i32],
) -> io::Result<()> {
    let mut trak_data = Vec::new();

    // Write tkhd (track header)
    write_tkhd(&mut trak_data, media_data, sample_sizes.len());

    // Write mdia (media)
    write_mdia(
        &mut trak_data,
        media_data,
        mdat_size,
        sample_sizes,
        composition_offsets,
    )?;

    write_box_header(buffer, "trak", (8 + trak_data.len()) as u32);
    buffer.extend_from_slice(&trak_data);

    Ok(())
}

fn write_tkhd(buffer: &mut Vec<u8>, media_data: &MediaData, sample_count: usize) {
    let duration = sample_count as u32 * 3000;
    let width = (media_data.width as u32) << 16;
    let height = (media_data.height as u32) << 16;

    let tkhd_data = vec![
        0x00,
        0x00,
        0x00,
        0x5C, // Box size (92 bytes)
        b't',
        b'k',
        b'h',
        b'd', // Box type
        0x00, // Version
        0x00,
        0x00,
        0x07, // Flags (track enabled, in movie, in preview)
        0x00,
        0x00,
        0x00,
        0x00, // Creation time
        0x00,
        0x00,
        0x00,
        0x00, // Modification time
        0x00,
        0x00,
        0x00,
        0x01, // Track ID
        0x00,
        0x00,
        0x00,
        0x00, // Reserved
        (duration >> 24) as u8,
        (duration >> 16) as u8,
        (duration >> 8) as u8,
        duration as u8,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Reserved
        0x00,
        0x00, // Layer
        0x00,
        0x00, // Alternate group
        0x00,
        0x00, // Volume
        0x00,
        0x00, // Reserved
        0x00,
        0x01,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x01,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x40,
        0x00,
        0x00,
        0x00, // Matrix
        (width >> 24) as u8,
        (width >> 16) as u8,
        (width >> 8) as u8,
        width as u8,
        (height >> 24) as u8,
        (height >> 16) as u8,
        (height >> 8) as u8,
        height as u8,
    ];

    buffer.extend_from_slice(&tkhd_data);
}

fn write_mdia(
    buffer: &mut Vec<u8>,
    media_data: &MediaData,
    mdat_size: usize,
    sample_sizes: &[u32],
    composition_offsets: &[i32],
) -> io::Result<()> {
    let mut mdia_data = Vec::new();

    // Write mdhd (media header)
    write_mdhd(&mut mdia_data, sample_sizes.len());

    // Write hdlr (handler)
    write_hdlr(&mut mdia_data);

    // Write minf (media information)
    write_minf(
        &mut mdia_data,
        media_data,
        mdat_size,
        sample_sizes,
        composition_offsets,
    )?;

    write_box_header(buffer, "mdia", (8 + mdia_data.len()) as u32);
    buffer.extend_from_slice(&mdia_data);

    Ok(())
}

fn write_mdhd(buffer: &mut Vec<u8>, sample_count: usize) {
    let timescale = 90000;
    let duration = sample_count as u32 * 3000;

    let mdhd_data = vec![
        0x00,
        0x00,
        0x00,
        0x20, // Box size (32 bytes)
        b'm',
        b'd',
        b'h',
        b'd', // Box type
        0x00, // Version
        0x00,
        0x00,
        0x00, // Flags
        0x00,
        0x00,
        0x00,
        0x00, // Creation time
        0x00,
        0x00,
        0x00,
        0x00, // Modification time
        (timescale >> 24) as u8,
        (timescale >> 16) as u8,
        (timescale >> 8) as u8,
        timescale as u8,
        (duration >> 24) as u8,
        (duration >> 16) as u8,
        (duration >> 8) as u8,
        duration as u8,
        0x55,
        0xC4, // Language (und = undetermined)
        0x00,
        0x00, // Pre-defined
    ];

    buffer.extend_from_slice(&mdhd_data);
}

fn write_hdlr(buffer: &mut Vec<u8>) {
    let hdlr_data = vec![
        0x00, 0x00, 0x00, 0x21, // Box size (33 bytes)
        b'h', b'd', b'l', b'r', // Box type
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, 0x00, 0x00, // Pre-defined
        b'v', b'i', b'd', b'e', // Handler type (video)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Reserved
        0x00, // Name (null terminated)
    ];

    buffer.extend_from_slice(&hdlr_data);
}

fn write_minf(
    buffer: &mut Vec<u8>,
    media_data: &MediaData,
    mdat_size: usize,
    sample_sizes: &[u32],
    composition_offsets: &[i32],
) -> io::Result<()> {
    let mut minf_data = Vec::new();

    // Write vmhd (video media header)
    write_vmhd(&mut minf_data);

    // Write dinf (data information)
    write_dinf(&mut minf_data);

    // Write stbl (sample table)
    write_stbl(
        &mut minf_data,
        media_data,
        mdat_size,
        sample_sizes,
        composition_offsets,
    )?;

    write_box_header(buffer, "minf", (8 + minf_data.len()) as u32);
    buffer.extend_from_slice(&minf_data);

    Ok(())
}

fn write_vmhd(buffer: &mut Vec<u8>) {
    let vmhd_data = vec![
        0x00, 0x00, 0x00, 0x14, // Box size (20 bytes)
        b'v', b'm', b'h', b'd', // Box type
        0x00, // Version
        0x00, 0x00, 0x01, // Flags
        0x00, 0x00, // Graphics mode
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Opcolor
    ];

    buffer.extend_from_slice(&vmhd_data);
}

fn write_dinf(buffer: &mut Vec<u8>) {
    let dref_data = vec![
        0x00, 0x00, 0x00, 0x1C, // dref box size
        b'd', b'r', b'e', b'f', // Box type
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, 0x00, 0x01, // Entry count
        0x00, 0x00, 0x00, 0x0C, // url box size
        b'u', b'r', b'l', b' ', // Box type
        0x00, // Version
        0x00, 0x00, 0x01, // Flags (self-reference)
    ];

    write_box_header(buffer, "dinf", (8 + dref_data.len()) as u32);
    buffer.extend_from_slice(&dref_data);
}

fn write_stbl(
    buffer: &mut Vec<u8>,
    media_data: &MediaData,
    _mdat_size: usize,
    sample_sizes: &[u32],
    composition_offsets: &[i32],
) -> io::Result<()> {
    let mut stbl_data = Vec::new();

    // Write stsd (sample description)
    write_stsd(&mut stbl_data, media_data);

    // Write stts (time-to-sample) - 30 fps = 3000 per frame at 90000 timescale
    let sample_count = sample_sizes.len() as u32;
    write_stts(&mut stbl_data, sample_count, 3000);

    // Write stsc (sample-to-chunk)
    write_stsc(&mut stbl_data);

    // Write stsz (sample sizes)
    write_stsz(&mut stbl_data, sample_sizes);

    // Write stco (chunk offsets)
    write_stco(&mut stbl_data, media_data, sample_sizes);

    // Write ctts (composition time to sample) - for B-frames
    write_ctts(&mut stbl_data, composition_offsets);

    write_box_header(buffer, "stbl", (8 + stbl_data.len()) as u32);
    buffer.extend_from_slice(&stbl_data);

    Ok(())
}

fn write_stsd(buffer: &mut Vec<u8>, media_data: &MediaData) {
    let width = media_data.width;
    let height = media_data.height;

    let mut avc1_data = vec![
        0x00,
        0x00,
        0x00,
        0x00, // box size placeholder (will update)
        b'a',
        b'v',
        b'c',
        b'1', // Box type
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Reserved
        0x00,
        0x01, // Data reference index
        0x00,
        0x00, // Pre-defined
        0x00,
        0x00, // Reserved
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Pre-defined
        (width >> 8) as u8,
        (width & 0xFF) as u8, // Width
        (height >> 8) as u8,
        (height & 0xFF) as u8, // Height
        0x00,
        0x48,
        0x00,
        0x00, // Horizontal resolution (72 dpi)
        0x00,
        0x48,
        0x00,
        0x00, // Vertical resolution (72 dpi)
        0x00,
        0x00,
        0x00,
        0x00, // Reserved
        0x00,
        0x01, // Frame count
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Compressor name
        0x00,
        0x18, // Depth
        0xFF,
        0xFF, // Pre-defined
    ];

    // Add avcC box if we have SPS/PPS
    if let (Some(sps), Some(pps)) = (&media_data.sps, &media_data.pps) {
        let mut avcc_data = vec![
            0x00, 0x00, 0x00, 0x00, // box size placeholder
            b'a', b'v', b'c', b'C', // Box type
            0x01, // Configuration version
        ];

        // Profile, profile compatibility, level from SPS
        if sps.len() >= 4 {
            avcc_data.push(sps[1]); // Profile
            avcc_data.push(sps[2]); // Profile compatibility
            avcc_data.push(sps[3]); // Level
        } else {
            avcc_data.extend_from_slice(&[0x64, 0x00, 0x1F]); // Default: High Profile, Level 3.1
        }

        avcc_data.push(0xFF); // 6 bits reserved (111111) + 2 bits nal size length - 1 (11)
        avcc_data.push(0xE1); // 3 bits reserved (111) + 5 bits number of SPS (00001)

        // SPS size and data
        avcc_data.extend_from_slice(&(sps.len() as u16).to_be_bytes());
        avcc_data.extend_from_slice(sps);

        // Number of PPS
        avcc_data.push(0x01);

        // PPS size and data
        avcc_data.extend_from_slice(&(pps.len() as u16).to_be_bytes());
        avcc_data.extend_from_slice(pps);

        // Update avcC box size
        let avcc_size = avcc_data.len() as u32;
        avcc_data[0..4].copy_from_slice(&avcc_size.to_be_bytes());

        avc1_data.extend_from_slice(&avcc_data);
    }

    // Update avc1 box size
    let avc1_size = avc1_data.len() as u32;
    avc1_data[0..4].copy_from_slice(&avc1_size.to_be_bytes());

    let mut stsd_data = Vec::new();
    stsd_data.extend_from_slice(&[
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, 0x00, 0x01, // Entry count
    ]);
    stsd_data.extend_from_slice(&avc1_data);

    write_box_header(buffer, "stsd", (8 + stsd_data.len()) as u32);
    buffer.extend_from_slice(&stsd_data);
}

fn write_stts(buffer: &mut Vec<u8>, sample_count: u32, sample_delta: u32) {
    let mut stts_data = Vec::new();
    stts_data.extend_from_slice(&[
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, 0x00, 0x01, // Entry count (1 entry for all samples)
    ]);
    stts_data.extend_from_slice(&sample_count.to_be_bytes());
    stts_data.extend_from_slice(&sample_delta.to_be_bytes());

    write_box_header(buffer, "stts", (8 + stts_data.len()) as u32);
    buffer.extend_from_slice(&stts_data);
}

fn write_stsc(buffer: &mut Vec<u8>) {
    let stsc_data = vec![
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, 0x00, 0x01, // Entry count
        0x00, 0x00, 0x00, 0x01, // First chunk
        0x00, 0x00, 0x00, 0x01, // Samples per chunk
        0x00, 0x00, 0x00, 0x01, // Sample description index
    ];

    write_box_header(buffer, "stsc", (8 + stsc_data.len()) as u32);
    buffer.extend_from_slice(&stsc_data);
}

fn write_stsz(buffer: &mut Vec<u8>, sample_sizes: &[u32]) {
    let sample_count = sample_sizes.len() as u32;

    let mut stsz_data = Vec::new();
    stsz_data.extend_from_slice(&[
        0x00, // Version
        0x00,
        0x00,
        0x00, // Flags
        0x00,
        0x00,
        0x00,
        0x00, // Sample size (0 = variable)
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8,
    ]);

    // Write individual sample sizes (actual AVCC sizes)
    for &size in sample_sizes {
        stsz_data.extend_from_slice(&size.to_be_bytes());
    }

    write_box_header(buffer, "stsz", (8 + stsz_data.len()) as u32);
    buffer.extend_from_slice(&stsz_data);
}

fn write_stco(buffer: &mut Vec<u8>, media_data: &MediaData, sample_sizes: &[u32]) {
    let sample_count = sample_sizes.len() as u32;

    // Calculate offset: ftyp + moov + mdat header
    let mut offset = 28; // ftyp size

    // Calculate exact moov size
    let moov_size = calculate_exact_moov_size(media_data, sample_sizes);
    offset += moov_size;
    offset += 8; // mdat header

    let mut stco_data = Vec::new();
    stco_data.extend_from_slice(&[
        0x00, // Version
        0x00,
        0x00,
        0x00, // Flags
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8,
    ]);

    // Write chunk offsets (one chunk per sample in this simple implementation)
    for &size in sample_sizes {
        stco_data.extend_from_slice(&(offset as u32).to_be_bytes());
        offset += size as usize;
    }

    write_box_header(buffer, "stco", (8 + stco_data.len()) as u32);
    buffer.extend_from_slice(&stco_data);
}

fn write_ctts(buffer: &mut Vec<u8>, composition_offsets: &[i32]) {
    // Check if we need ctts (if all offsets are 0, we can skip it)
    let has_non_zero = composition_offsets.iter().any(|&offset| offset != 0);

    if !has_non_zero {
        return; // No B-frames, no need for ctts
    }

    let sample_count = composition_offsets.len() as u32;

    let mut ctts_data = Vec::new();
    ctts_data.extend_from_slice(&[
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
    ]);

    // Write entries (we could compress runs of same values, but keep it simple)
    ctts_data.extend_from_slice(&sample_count.to_be_bytes()); // Entry count

    for &offset in composition_offsets {
        ctts_data.extend_from_slice(&1u32.to_be_bytes()); // Sample count (1)
        ctts_data.extend_from_slice(&offset.to_be_bytes()); // Sample offset
    }

    write_box_header(buffer, "ctts", (8 + ctts_data.len()) as u32);
    buffer.extend_from_slice(&ctts_data);
}

fn write_simple_stco(buffer: &mut Vec<u8>) {
    // Simple stco with single chunk offset (placeholder)
    let stco_data = vec![
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, 0x00, 0x01, // Entry count (1 chunk)
        0x00, 0x00, 0x00, 0x00, // Offset (will be wrong but harmless)
    ];

    write_box_header(buffer, "stco", (8 + stco_data.len()) as u32);
    buffer.extend_from_slice(&stco_data);
}

fn write_audio_stco(buffer: &mut Vec<u8>, sample_sizes: &[u32], audio_mdat_offset: usize) {
    let sample_count = sample_sizes.len() as u32;

    // Calculate offset: ftyp + moov + mdat header + audio offset in mdat
    let mut offset = 28; // ftyp size

    // We'll calculate moov size later, use placeholder for now
    // This is a simplified version - in production you'd need exact calculation
    offset += 2000; // Approximate moov size (will update this)
    offset += 8; // mdat header
    offset += audio_mdat_offset; // Offset to audio data within mdat

    let mut stco_data = Vec::new();
    stco_data.extend_from_slice(&[
        0x00, // Version
        0x00,
        0x00,
        0x00, // Flags
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8,
    ]);

    // Write chunk offsets (one chunk per sample)
    for &size in sample_sizes {
        stco_data.extend_from_slice(&(offset as u32).to_be_bytes());
        offset += size as usize;
    }

    write_box_header(buffer, "stco", (8 + stco_data.len()) as u32);
    buffer.extend_from_slice(&stco_data);
}

fn calculate_exact_moov_size(media_data: &MediaData, sample_sizes: &[u32]) -> usize {
    let sample_count = sample_sizes.len();

    // Calculate exact avcC size
    let avcc_size = if let (Some(sps), Some(pps)) = (&media_data.sps, &media_data.pps) {
        8 + // box header
        1 + // version
        3 + // profile/level
        2 + // flags + num SPS
        2 + sps.len() + // SPS length + data
        1 + // num PPS  
        2 + pps.len() // PPS length + data
    } else {
        0
    };

    // Calculate exact sizes
    let mvhd_size = 108;
    let tkhd_size = 92;
    let mdhd_size = 32;
    let hdlr_size = 33;
    let vmhd_size = 20;
    let dinf_size = 36;
    let avc1_base_size = 86;
    let stsd_size = 8 + 4 + 4 + avc1_base_size + avcc_size;
    let stts_size = 8 + 4 + 4 + 8;
    let stsc_size = 8 + 4 + 4 + 12;
    let stsz_size = 8 + 4 + 4 + 4 + (sample_count * 4);
    let stco_size = 8 + 4 + 4 + (sample_count * 4);
    let ctts_size = 8 + 4 + 4 + (sample_count * 8); // Each entry: 4 bytes count + 4 bytes offset

    let stbl_size = 8 + stsd_size + stts_size + stsc_size + stsz_size + stco_size + ctts_size;
    let minf_size = 8 + vmhd_size + dinf_size + stbl_size;
    let mdia_size = 8 + mdhd_size + hdlr_size + minf_size;
    let trak_size = 8 + tkhd_size + mdia_size;
    let moov_size = 8 + mvhd_size + trak_size;

    moov_size
}

fn write_audio_trak(
    buffer: &mut Vec<u8>,
    sample_sizes: &[u32],
    audio_mdat_offset: usize,
) -> io::Result<()> {
    let mut trak_data = Vec::new();

    // Write tkhd (track header) for audio
    write_audio_tkhd(&mut trak_data, sample_sizes.len());

    // Write mdia (media)
    write_audio_mdia(&mut trak_data, sample_sizes, audio_mdat_offset)?;

    write_box_header(buffer, "trak", (8 + trak_data.len()) as u32);
    buffer.extend_from_slice(&trak_data);

    Ok(())
}

fn write_audio_tkhd(buffer: &mut Vec<u8>, sample_count: usize) {
    let timescale = 48000; // 48kHz audio
    let duration = sample_count as u32 * 1024; // AAC frame size

    let tkhd_data = vec![
        0x00,
        0x00,
        0x00,
        0x5C, // Box size (92 bytes)
        b't',
        b'k',
        b'h',
        b'd', // Box type
        0x00, // Version
        0x00,
        0x00,
        0x07, // Flags (track enabled, in movie, in preview)
        0x00,
        0x00,
        0x00,
        0x00, // Creation time
        0x00,
        0x00,
        0x00,
        0x00, // Modification time
        0x00,
        0x00,
        0x00,
        0x02, // Track ID (2 for audio)
        0x00,
        0x00,
        0x00,
        0x00, // Reserved
        (duration >> 24) as u8,
        (duration >> 16) as u8,
        (duration >> 8) as u8,
        duration as u8, // Duration
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Reserved
        0x00,
        0x00, // Layer
        0x00,
        0x00, // Alternate group
        0x01,
        0x00, // Volume (1.0)
        0x00,
        0x00, // Reserved
        0x00,
        0x01,
        0x00,
        0x00, // Matrix
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x01,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x40,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // Width
        0x00,
        0x00,
        0x00,
        0x00, // Height
    ];

    buffer.extend_from_slice(&tkhd_data);
}

fn write_audio_mdia(
    buffer: &mut Vec<u8>,
    sample_sizes: &[u32],
    audio_mdat_offset: usize,
) -> io::Result<()> {
    let mut mdia_data = Vec::new();

    // Write mdhd (media header)
    write_audio_mdhd(&mut mdia_data, sample_sizes.len());

    // Write hdlr (handler)
    write_audio_hdlr(&mut mdia_data);

    // Write minf (media information)
    write_audio_minf(&mut mdia_data, sample_sizes, audio_mdat_offset)?;

    write_box_header(buffer, "mdia", (8 + mdia_data.len()) as u32);
    buffer.extend_from_slice(&mdia_data);

    Ok(())
}

fn write_audio_mdhd(buffer: &mut Vec<u8>, sample_count: usize) {
    let timescale = 48000u32; // 48kHz
    let duration = sample_count as u32 * 1024; // AAC frame = 1024 samples

    let mdhd_data = vec![
        0x00,
        0x00,
        0x00,
        0x20, // Box size (32 bytes)
        b'm',
        b'd',
        b'h',
        b'd', // Box type
        0x00, // Version
        0x00,
        0x00,
        0x00, // Flags
        0x00,
        0x00,
        0x00,
        0x00, // Creation time
        0x00,
        0x00,
        0x00,
        0x00, // Modification time
        (timescale >> 24) as u8,
        (timescale >> 16) as u8,
        (timescale >> 8) as u8,
        timescale as u8,
        (duration >> 24) as u8,
        (duration >> 16) as u8,
        (duration >> 8) as u8,
        duration as u8,
        0x55,
        0xC4, // Language (und)
        0x00,
        0x00, // Pre-defined
    ];

    buffer.extend_from_slice(&mdhd_data);
}

fn write_audio_hdlr(buffer: &mut Vec<u8>) {
    let hdlr_data = vec![
        0x00, 0x00, 0x00, 0x21, // Box size (33 bytes)
        b'h', b'd', b'l', b'r', // Box type
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, 0x00, 0x00, // Pre-defined
        b's', b'o', b'u', b'n', // Handler type (sound)
        0x00, 0x00, 0x00, 0x00, // Reserved
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Name (empty)
    ];

    buffer.extend_from_slice(&hdlr_data);
}

fn write_audio_minf(
    buffer: &mut Vec<u8>,
    sample_sizes: &[u32],
    audio_mdat_offset: usize,
) -> io::Result<()> {
    let mut minf_data = Vec::new();

    // Write smhd (sound media header)
    write_smhd(&mut minf_data);

    // Write dinf (data information)
    write_dinf(&mut minf_data);

    // Write stbl (sample table)
    write_audio_stbl(&mut minf_data, sample_sizes, audio_mdat_offset)?;

    write_box_header(buffer, "minf", (8 + minf_data.len()) as u32);
    buffer.extend_from_slice(&minf_data);

    Ok(())
}

fn write_smhd(buffer: &mut Vec<u8>) {
    let smhd_data = vec![
        0x00, 0x00, 0x00, 0x10, // Box size (16 bytes)
        b's', b'm', b'h', b'd', // Box type
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, // Balance
        0x00, 0x00, // Reserved
    ];

    buffer.extend_from_slice(&smhd_data);
}

fn write_audio_stbl(
    buffer: &mut Vec<u8>,
    sample_sizes: &[u32],
    audio_mdat_offset: usize,
) -> io::Result<()> {
    let mut stbl_data = Vec::new();

    // Write stsd (sample description) for audio
    write_audio_stsd(&mut stbl_data);

    // Write stts (time-to-sample) - AAC: 1024 samples per frame at 48000 Hz
    let sample_count = sample_sizes.len() as u32;
    write_stts(&mut stbl_data, sample_count, 1024);

    // Write stsc (sample-to-chunk)
    write_stsc(&mut stbl_data);

    // Write stsz (sample sizes)
    write_stsz(&mut stbl_data, sample_sizes);

    // Write stco (chunk offsets) - need proper offsets for audio
    write_audio_stco(&mut stbl_data, sample_sizes, audio_mdat_offset);

    write_box_header(buffer, "stbl", (8 + stbl_data.len()) as u32);
    buffer.extend_from_slice(&stbl_data);

    Ok(())
}

fn write_audio_stsd(buffer: &mut Vec<u8>) {
    // esds box (Elementary Stream Descriptor)
    let mut esds_content = Vec::new();
    esds_content.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x00, // Version + Flags
        0x03, // ES_DescrTag
        0x19, // Length (25 bytes)
        0x00, 0x00, // ES_ID
        0x00, // Flags
        0x04, // DecoderConfigDescrTag
        0x11, // Length (17 bytes)
        0x40, // Object type (AAC)
        0x15, // Stream type (Audio = 5) | upstream (0)
        0x00, 0x03, 0x00, // Buffer size (768)
        0x00, 0x00, 0xFA, 0x00, // Max bitrate (64000)
        0x00, 0x00, 0xFA, 0x00, // Avg bitrate (64000)
        0x05, // DecoderSpecificInfoTag
        0x02, // Length (2 bytes)
        0x11, 0x90, // AAC-LC (2), 48kHz (3), stereo (2)
        0x06, // SLConfigDescrTag
        0x01, // Length (1 byte)
        0x02, // Reserved
    ]);

    // mp4a box (MPEG-4 Audio)
    let mut mp4a_content = Vec::new();
    mp4a_content.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Reserved
        0x00, 0x01, // Data reference index
        0x00, 0x00, 0x00, 0x00, // Version + flags (audio)
        0x00, 0x00, 0x00, 0x00, // Reserved
        0x00, 0x02, // Channel count (2 = stereo)
        0x00, 0x10, // Sample size (16 bits)
        0x00, 0x00, // Pre-defined
        0x00, 0x00, // Reserved
        0xBB, 0x80, 0x00, 0x00, // Sample rate (48000 Hz in 16.16 fixed point)
    ]);

    // Add esds to mp4a
    write_box_header(&mut mp4a_content, "esds", (8 + esds_content.len()) as u32);
    mp4a_content.extend_from_slice(&esds_content);

    // stsd box
    let mut stsd_data = Vec::new();
    stsd_data.extend_from_slice(&[
        0x00, // Version
        0x00, 0x00, 0x00, // Flags
        0x00, 0x00, 0x00, 0x01, // Entry count
    ]);

    write_box_header(&mut stsd_data, "mp4a", (8 + mp4a_content.len()) as u32);
    stsd_data.extend_from_slice(&mp4a_content);

    write_box_header(buffer, "stsd", (8 + stsd_data.len()) as u32);
    buffer.extend_from_slice(&stsd_data);
}
