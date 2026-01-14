use std::io::{self, ErrorKind};

const TS_PACKET_SIZE: usize = 188;
const SYNC_BYTE: u8 = 0x47;

#[derive(Debug)]
pub struct MediaData {
    pub video_stream: Vec<u8>, // Combined video stream
    pub frame_timestamps: Vec<(Option<u64>, Option<u64>)>, // (PTS, DTS) pairs
    pub video_pid: Option<u16>,
    pub audio_pid: Option<u16>,
    pub audio_frames: Vec<Vec<u8>>, // AAC audio frames (without ADTS headers)
    pub audio_timestamps: Vec<Option<u64>>, // Audio PTS values
    pub audio_buffer: Vec<u8>,      // Temporary buffer for collecting audio PES packets
    pub current_audio_pts: Option<u64>, // PTS for the current audio PES packet being accumulated
    pub width: u16,
    pub height: u16,
    pub sps: Option<Vec<u8>>,
    pub pps: Option<Vec<u8>>,
}

impl MediaData {
    pub fn new() -> Self {
        MediaData {
            video_stream: Vec::new(),
            frame_timestamps: Vec::new(),
            video_pid: None,
            audio_pid: None,
            audio_frames: Vec::new(),
            audio_timestamps: Vec::new(),
            audio_buffer: Vec::new(),
            current_audio_pts: None,
            width: 1920,
            height: 1080,
            sps: None,
            pps: None,
        }
    }
}

pub fn parse_ts_packets(data: &[u8]) -> io::Result<MediaData> {
    let mut media_data = MediaData::new();
    let mut offset = 0;

    // Find first sync byte
    while offset < data.len() && data[offset] != SYNC_BYTE {
        offset += 1;
    }

    if offset >= data.len() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "No valid TS sync byte found",
        ));
    }

    let mut pat_pmt_parsed = false;
    let mut pmt_pid: Option<u16> = None;

    while offset + TS_PACKET_SIZE <= data.len() {
        let packet = &data[offset..offset + TS_PACKET_SIZE];

        if packet[0] != SYNC_BYTE {
            // Try to resync
            offset += 1;
            while offset < data.len() && data[offset] != SYNC_BYTE {
                offset += 1;
            }
            continue;
        }

        // Parse TS header
        let pid = ((packet[1] as u16 & 0x1F) << 8) | (packet[2] as u16);
        let payload_start = (packet[1] & 0x40) != 0;
        let has_adaptation = (packet[3] & 0x20) != 0;
        let has_payload = (packet[3] & 0x10) != 0;

        let mut payload_offset = 4;

        // Skip adaptation field
        if has_adaptation {
            let adaptation_len = packet[4] as usize;
            payload_offset += 1 + adaptation_len;
        }

        if !has_payload || payload_offset >= TS_PACKET_SIZE {
            offset += TS_PACKET_SIZE;
            continue;
        }

        let payload = &packet[payload_offset..];

        // Parse PAT (PID 0)
        if pid == 0 && !pat_pmt_parsed {
            if let Some(pmt) = parse_pat(payload, payload_start) {
                pmt_pid = Some(pmt);
            }
        }
        // Parse PMT
        else if Some(pid) == pmt_pid && !pat_pmt_parsed {
            if let Some((vpid, apid)) = parse_pmt(payload, payload_start) {
                media_data.video_pid = Some(vpid);
                media_data.audio_pid = Some(apid);
                pat_pmt_parsed = true;
            }
        }
        // Collect media packets
        else if Some(pid) == media_data.video_pid {
            if payload_start && payload.len() >= 9 {
                // PES packet - extract timestamps
                let (pts, dts) = extract_pes_timestamps(payload);
                media_data.frame_timestamps.push((pts, dts));

                let pes_data = extract_pes_payload(payload);
                if !pes_data.is_empty() {
                    // Check for SPS/PPS NAL units
                    extract_h264_params(&mut media_data, &pes_data);
                    // Append to video stream
                    media_data.video_stream.extend_from_slice(&pes_data);
                }
            } else if !payload_start && !payload.is_empty() {
                // Continuation of PES packet - append directly
                media_data.video_stream.extend_from_slice(payload);
            }
        } else if Some(pid) == media_data.audio_pid {
            if payload_start && payload.len() >= 9 {
                // Process any buffered audio data from previous PES packet
                if !media_data.audio_buffer.is_empty() {
                    let (aac_frames, consumed) = extract_aac_frames(&media_data.audio_buffer);
                    // Use the stored PTS from the previous PES packet
                    for _ in 0..aac_frames.len() {
                        media_data
                            .audio_timestamps
                            .push(media_data.current_audio_pts);
                    }
                    media_data.audio_frames.extend(aac_frames);
                    // Remove consumed bytes from buffer
                    media_data.audio_buffer.drain(..consumed);
                }

                // Start of new audio PES packet - extract timestamps and data
                let (pts, _) = extract_pes_timestamps(payload);
                media_data.current_audio_pts = pts; // Store PTS for this PES packet

                let pes_data = extract_pes_payload(payload);

                // Store in buffer for potential continuation packets
                media_data.audio_buffer.extend_from_slice(&pes_data);

                // Try to extract complete AAC frames
                let (aac_frames, consumed) = extract_aac_frames(&media_data.audio_buffer);
                if !aac_frames.is_empty() {
                    // Assign PTS to all frames extracted from this PES packet
                    for _ in 0..aac_frames.len() {
                        media_data.audio_timestamps.push(pts);
                    }
                    media_data.audio_frames.extend(aac_frames);
                    // Remove consumed bytes from buffer
                    media_data.audio_buffer.drain(..consumed);
                }
                // If no complete frames, keep data in buffer for continuation
            } else if !payload_start && !payload.is_empty() {
                // Continuation of audio PES packet
                media_data.audio_buffer.extend_from_slice(payload);

                // Try to extract complete AAC frames from accumulated data
                let (aac_frames, consumed) = extract_aac_frames(&media_data.audio_buffer);
                if !aac_frames.is_empty() {
                    // Use the PTS stored from the PES packet start
                    for _ in 0..aac_frames.len() {
                        media_data
                            .audio_timestamps
                            .push(media_data.current_audio_pts);
                    }
                    media_data.audio_frames.extend(aac_frames);
                    // Remove consumed bytes from buffer
                    media_data.audio_buffer.drain(..consumed);
                }
            }
        }

        offset += TS_PACKET_SIZE;
    }

    // Process any remaining buffered audio data
    if !media_data.audio_buffer.is_empty() {
        let (aac_frames, _consumed) = extract_aac_frames(&media_data.audio_buffer);
        for _ in 0..aac_frames.len() {
            media_data
                .audio_timestamps
                .push(media_data.current_audio_pts);
        }
        media_data.audio_frames.extend(aac_frames);
    }

    println!(
        "Total audio frames collected: {}",
        media_data.audio_frames.len()
    );
    println!(
        "Total video frames collected: {}",
        media_data.frame_timestamps.len()
    );

    if !media_data.audio_timestamps.is_empty() {
        if let Some(Some(first_audio_pts)) = media_data.audio_timestamps.first() {
            if let Some(Some(last_audio_pts)) = media_data.audio_timestamps.last() {
                println!(
                    "Audio PTS range: {} - {} ({:.2} - {:.2} sec)",
                    first_audio_pts,
                    last_audio_pts,
                    *first_audio_pts as f64 / 90000.0,
                    *last_audio_pts as f64 / 90000.0
                );
            }
        }
    }

    if !media_data.frame_timestamps.is_empty() {
        if let Some(&(Some(first_video_pts), _)) = media_data.frame_timestamps.first() {
            if let Some(&(Some(last_video_pts), _)) = media_data.frame_timestamps.last() {
                println!(
                    "Video PTS range: {} - {} ({:.2} - {:.2} sec)",
                    first_video_pts,
                    last_video_pts,
                    first_video_pts as f64 / 90000.0,
                    last_video_pts as f64 / 90000.0
                );
            }
        }
    }

    Ok(media_data)
}

fn parse_pat(payload: &[u8], payload_start: bool) -> Option<u16> {
    if !payload_start || payload.len() < 13 {
        return None;
    }

    let mut offset = 0;

    // Skip pointer field
    if payload_start {
        offset += payload[0] as usize + 1;
    }

    if offset + 12 >= payload.len() {
        return None;
    }

    // Parse PAT
    let table_id = payload[offset];
    if table_id != 0x00 {
        return None;
    }

    // Get PMT PID from first program
    let pmt_pid = ((payload[offset + 10] as u16 & 0x1F) << 8) | (payload[offset + 11] as u16);

    Some(pmt_pid)
}

fn parse_pmt(payload: &[u8], payload_start: bool) -> Option<(u16, u16)> {
    if !payload_start || payload.len() < 16 {
        return None;
    }

    let mut offset = 0;

    // Skip pointer field
    if payload_start {
        offset += payload[0] as usize + 1;
    }

    if offset + 12 >= payload.len() {
        return None;
    }

    let table_id = payload[offset];
    if table_id != 0x02 {
        return None;
    }

    let section_length =
        (((payload[offset + 1] as u16 & 0x0F) << 8) | payload[offset + 2] as u16) as usize;
    let program_info_length =
        (((payload[offset + 10] as u16 & 0x0F) << 8) | payload[offset + 11] as u16) as usize;

    offset += 12 + program_info_length;

    let mut video_pid: Option<u16> = None;
    let mut audio_pid: Option<u16> = None;

    // Parse stream descriptors
    while offset + 5 <= payload.len() && offset < section_length + 3 {
        let stream_type = payload[offset];
        let elementary_pid =
            ((payload[offset + 1] as u16 & 0x1F) << 8) | (payload[offset + 2] as u16);
        let es_info_length =
            (((payload[offset + 3] as u16 & 0x0F) << 8) | payload[offset + 4] as u16) as usize;

        // H.264 video
        if stream_type == 0x1B && video_pid.is_none() {
            video_pid = Some(elementary_pid);
        }
        // AAC audio
        else if stream_type == 0x0F && audio_pid.is_none() {
            audio_pid = Some(elementary_pid);
        }
        // MPEG-2 video
        else if stream_type == 0x02 && video_pid.is_none() {
            video_pid = Some(elementary_pid);
        }
        // MPEG audio
        else if stream_type == 0x03 && audio_pid.is_none() {
            audio_pid = Some(elementary_pid);
        }

        offset += 5 + es_info_length;
    }

    if let (Some(v), Some(a)) = (video_pid, audio_pid) {
        println!("Found PIDs - Video: {}, Audio: {}", v, a);
        return Some((v, a));
    }

    None
}

fn extract_pes_payload(payload: &[u8]) -> Vec<u8> {
    if payload.len() < 9 {
        return Vec::new();
    }

    // Check PES start code
    if payload[0] != 0x00 || payload[1] != 0x00 || payload[2] != 0x01 {
        return Vec::new();
    }

    let pes_header_length = payload[8] as usize;
    let payload_start = 9 + pes_header_length;

    if payload_start >= payload.len() {
        return Vec::new();
    }

    payload[payload_start..].to_vec()
}

fn extract_pes_timestamps(payload: &[u8]) -> (Option<u64>, Option<u64>) {
    if payload.len() < 9 {
        return (None, None);
    }

    // Check PES start code
    if payload[0] != 0x00 || payload[1] != 0x00 || payload[2] != 0x01 {
        return (None, None);
    }

    let pts_dts_flags = (payload[7] >> 6) & 0x03;
    let pes_header_length = payload[8] as usize;

    if 9 + pes_header_length > payload.len() {
        return (None, None);
    }

    let mut pts = None;
    let mut dts = None;

    // PTS present
    if pts_dts_flags >= 2 && pes_header_length >= 5 {
        let pts_bytes = &payload[9..14];
        pts = Some(
            (((pts_bytes[0] as u64 & 0x0E) << 29)
                | ((pts_bytes[1] as u64) << 22)
                | ((pts_bytes[2] as u64 & 0xFE) << 14)
                | ((pts_bytes[3] as u64) << 7)
                | ((pts_bytes[4] as u64 & 0xFE) >> 1)) as u64,
        );
    }

    // DTS present
    if pts_dts_flags == 3 && pes_header_length >= 10 {
        let dts_bytes = &payload[14..19];
        dts = Some(
            (((dts_bytes[0] as u64 & 0x0E) << 29)
                | ((dts_bytes[1] as u64) << 22)
                | ((dts_bytes[2] as u64 & 0xFE) << 14)
                | ((dts_bytes[3] as u64) << 7)
                | ((dts_bytes[4] as u64 & 0xFE) >> 1)) as u64,
        );
    }

    (pts, dts)
}

fn extract_h264_params(media_data: &mut MediaData, pes_data: &[u8]) {
    let mut i = 0;

    while i + 4 <= pes_data.len() {
        let start_code_len = if i + 3 < pes_data.len()
            && pes_data[i] == 0x00
            && pes_data[i + 1] == 0x00
            && pes_data[i + 2] == 0x00
            && pes_data[i + 3] == 0x01
        {
            4
        } else if i + 2 < pes_data.len()
            && pes_data[i] == 0x00
            && pes_data[i + 1] == 0x00
            && pes_data[i + 2] == 0x01
        {
            3
        } else {
            i += 1;
            continue;
        };

        let nal_start = i + start_code_len;
        if nal_start >= pes_data.len() {
            break;
        }

        let nal_type = pes_data[nal_start] & 0x1F;

        // Find next NAL unit start code
        let mut nal_end = nal_start + 1;
        while nal_end + 2 < pes_data.len() {
            if pes_data[nal_end] == 0x00 && pes_data[nal_end + 1] == 0x00 {
                if nal_end + 2 < pes_data.len() && pes_data[nal_end + 2] == 0x01 {
                    break;
                } else if nal_end + 3 < pes_data.len()
                    && pes_data[nal_end + 2] == 0x00
                    && pes_data[nal_end + 3] == 0x01
                {
                    break;
                }
            }
            nal_end += 1;
        }

        if nal_end > pes_data.len() {
            nal_end = pes_data.len();
        }

        // SPS (Sequence Parameter Set)
        if nal_type == 7 && media_data.sps.is_none() {
            let sps_data = pes_data[nal_start..nal_end].to_vec();
            if let Some((w, h)) = parse_sps_resolution(&sps_data) {
                media_data.width = w;
                media_data.height = h;
            }
            media_data.sps = Some(sps_data);
        }
        // PPS (Picture Parameter Set)
        else if nal_type == 8 && media_data.pps.is_none() {
            let pps_data = pes_data[nal_start..nal_end].to_vec();
            media_data.pps = Some(pps_data);
        }

        i = nal_end;
    }
}

fn parse_sps_resolution(sps: &[u8]) -> Option<(u16, u16)> {
    // SPS structure (simplified):
    // - NAL header (1 byte)
    // - profile_idc (1 byte)
    // - constraints (1 byte)
    // - level_idc (1 byte)
    // - seq_parameter_set_id (ue(v))
    // - ... (depends on profile)
    // - pic_width_in_mbs_minus1 (ue(v))
    // - pic_height_in_map_units_minus1 (ue(v))

    if sps.len() < 4 {
        return None;
    }

    let profile_idc = sps[1];

    // Initialize bitstream reader
    let mut bit_reader = BitReader::new(&sps[4..]); // Skip NAL header + profile + constraint + level

    // Read seq_parameter_set_id
    if bit_reader.read_ue().is_none() {
        return None;
    }

    // Profile-specific fields
    if profile_idc == 100
        || profile_idc == 110
        || profile_idc == 122
        || profile_idc == 244
        || profile_idc == 44
        || profile_idc == 83
        || profile_idc == 86
        || profile_idc == 118
        || profile_idc == 128
    {
        // chroma_format_idc
        let chroma_format_idc = bit_reader.read_ue()?;

        if chroma_format_idc == 3 {
            // separate_colour_plane_flag
            bit_reader.read_bit()?;
        }

        // bit_depth_luma_minus8
        bit_reader.read_ue()?;
        // bit_depth_chroma_minus8
        bit_reader.read_ue()?;
        // qpprime_y_zero_transform_bypass_flag
        bit_reader.read_bit()?;

        // seq_scaling_matrix_present_flag
        if bit_reader.read_bit()? {
            let count = if chroma_format_idc != 3 { 8 } else { 12 };
            for _ in 0..count {
                if bit_reader.read_bit()? {
                    // Skip scaling list
                    let size = if count < 6 { 16 } else { 64 };
                    let mut last_scale = 8;
                    let mut next_scale = 8;
                    for _ in 0..size {
                        if next_scale != 0 {
                            let delta_scale = bit_reader.read_se()?;
                            next_scale = (last_scale + delta_scale + 256) % 256;
                        }
                        last_scale = if next_scale == 0 {
                            last_scale
                        } else {
                            next_scale
                        };
                    }
                }
            }
        }
    }

    // log2_max_frame_num_minus4
    bit_reader.read_ue()?;

    // pic_order_cnt_type
    let pic_order_cnt_type = bit_reader.read_ue()?;

    if pic_order_cnt_type == 0 {
        // log2_max_pic_order_cnt_lsb_minus4
        bit_reader.read_ue()?;
    } else if pic_order_cnt_type == 1 {
        // delta_pic_order_always_zero_flag
        bit_reader.read_bit()?;
        // offset_for_non_ref_pic
        bit_reader.read_se()?;
        // offset_for_top_to_bottom_field
        bit_reader.read_se()?;

        let num_ref_frames_in_pic_order_cnt_cycle = bit_reader.read_ue()?;
        for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
            // offset_for_ref_frame[i]
            bit_reader.read_se()?;
        }
    }

    // max_num_ref_frames
    bit_reader.read_ue()?;
    // gaps_in_frame_num_value_allowed_flag
    bit_reader.read_bit()?;

    // pic_width_in_mbs_minus1
    let pic_width_in_mbs_minus1 = bit_reader.read_ue()?;
    // pic_height_in_map_units_minus1
    let pic_height_in_map_units_minus1 = bit_reader.read_ue()?;

    // frame_mbs_only_flag
    let frame_mbs_only_flag = bit_reader.read_bit()?;

    let mut frame_crop_left = 0;
    let mut frame_crop_right = 0;
    let mut frame_crop_top = 0;
    let mut frame_crop_bottom = 0;

    if !frame_mbs_only_flag {
        // mb_adaptive_frame_field_flag
        bit_reader.read_bit()?;
    }

    // direct_8x8_inference_flag
    bit_reader.read_bit()?;

    // frame_cropping_flag
    if bit_reader.read_bit()? {
        frame_crop_left = bit_reader.read_ue()?;
        frame_crop_right = bit_reader.read_ue()?;
        frame_crop_top = bit_reader.read_ue()?;
        frame_crop_bottom = bit_reader.read_ue()?;
    }

    // Calculate actual width and height
    let width = ((pic_width_in_mbs_minus1 + 1) * 16) - (frame_crop_left + frame_crop_right) * 2;
    let height =
        ((2 - if frame_mbs_only_flag { 1 } else { 0 }) * (pic_height_in_map_units_minus1 + 1) * 16)
            - (frame_crop_top + frame_crop_bottom) * 2;

    Some((width as u16, height as u16))
}

// Bitstream reader for exponential-Golomb coding
struct BitReader<'a> {
    data: &'a [u8],
    byte_offset: usize,
    bit_offset: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader {
            data,
            byte_offset: 0,
            bit_offset: 0,
        }
    }

    fn read_bit(&mut self) -> Option<bool> {
        if self.byte_offset >= self.data.len() {
            return None;
        }

        let bit = (self.data[self.byte_offset] >> (7 - self.bit_offset)) & 1;
        self.bit_offset += 1;

        if self.bit_offset == 8 {
            self.bit_offset = 0;
            self.byte_offset += 1;
        }

        Some(bit != 0)
    }

    // Read unsigned exponential-Golomb code
    fn read_ue(&mut self) -> Option<u32> {
        let mut leading_zeros = 0;

        while !self.read_bit()? {
            leading_zeros += 1;
            if leading_zeros > 31 {
                return None; // Prevent overflow
            }
        }

        if leading_zeros == 0 {
            return Some(0);
        }

        let mut value = 1u32;
        for _ in 0..leading_zeros {
            value = (value << 1) | (if self.read_bit()? { 1 } else { 0 });
        }

        Some(value - 1)
    }

    // Read signed exponential-Golomb code
    fn read_se(&mut self) -> Option<i32> {
        let code = self.read_ue()?;
        let value = if code % 2 == 0 {
            -((code / 2) as i32)
        } else {
            ((code + 1) / 2) as i32
        };
        Some(value)
    }
}

fn extract_aac_frames(pes_payload: &[u8]) -> (Vec<Vec<u8>>, usize) {
    let mut frames = Vec::new();
    let mut offset = 0;
    let mut last_complete_offset = 0;

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
            // Incomplete frame - stop here
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
        last_complete_offset = offset;
    }

    (frames, last_complete_offset)
}
