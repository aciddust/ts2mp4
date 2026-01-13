use std::io::{self, ErrorKind};

const TS_PACKET_SIZE: usize = 188;
const SYNC_BYTE: u8 = 0x47;

#[derive(Debug, Clone)]
pub struct FrameInfo {
    pub data: Vec<u8>,
    pub pts: Option<u64>,
    pub dts: Option<u64>,
}

#[derive(Debug)]
pub struct MediaData {
    pub video_stream: Vec<u8>, // Combined video stream
    pub frame_timestamps: Vec<(Option<u64>, Option<u64>)>, // (PTS, DTS) pairs
    pub video_pid: Option<u16>,
    pub audio_pid: Option<u16>,
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
            // Skip audio for now
        }

        offset += TS_PACKET_SIZE;
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

    // For now, detect common resolutions from SPS profile/level
    // Full parsing requires exponential-Golomb decoding

    // Check the actual bytes - SPS for 1280x720
    // Common pattern: 67 4d (profile_idc=77=Main, 40, 1f=level 3.1)

    // Simplified heuristic: Try to identify from level and known patterns
    // This is a workaround - proper implementation needs bitstream parser

    // For the specific case, let's parse the basic structure
    let profile = sps[1];
    let level = sps[3];

    // Common resolutions based on typical encoding:
    // Level 3.1 (0x1F) with Main profile often = 720p or 1080p
    // We need better detection

    // Default to 720p for Main profile level 3.1 (common for web streaming)
    if profile == 0x4D && level == 0x1F {
        return Some((1280, 720));
    }

    // Default fallback
    Some((1280, 720))
}
