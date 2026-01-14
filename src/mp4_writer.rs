use crate::ts_parser::MediaData;
use std::io::{self, ErrorKind};

pub fn create_mp4(media_data: MediaData) -> io::Result<Vec<u8>> {
    if media_data.video_stream.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "No video data found",
        ));
    }

    // Step 1: Prepare video data
    let frames = split_into_frames(&media_data.video_stream);
    let mut video_samples = Vec::new();

    for frame in frames.iter() {
        let sample_data = convert_annexb_to_avcc(frame);
        if !sample_data.is_empty() {
            video_samples.push(sample_data);
        }
    }

    // Step 2: Prepare audio data (already in correct format)
    let audio_samples = &media_data.audio_frames;

    // Step 3: Build mdat
    let mut mdat_data = Vec::new();
    for sample in &video_samples {
        mdat_data.extend_from_slice(sample);
    }
    let video_data_end = mdat_data.len();

    for sample in audio_samples {
        mdat_data.extend_from_slice(sample);
    }

    // Step 4: Calculate offsets
    let ftyp_size = 28;
    let mdat_header_size = 8;

    // Build moov to calculate its size
    let moov_box = build_moov(
        &media_data,
        &video_samples,
        audio_samples,
        ftyp_size,
        0, // moov_size placeholder
        mdat_header_size,
        video_data_end,
    )?;

    let moov_size = moov_box.len();

    // Rebuild moov with correct offsets
    let moov_box = build_moov(
        &media_data,
        &video_samples,
        audio_samples,
        ftyp_size,
        moov_size,
        mdat_header_size,
        video_data_end,
    )?;

    // Step 5: Write MP4 file
    let mut mp4_buffer = Vec::new();

    // ftyp
    mp4_buffer.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x1C, // size
        b'f', b't', b'y', b'p', b'i', b's', b'o', b'm', 0x00, 0x00, 0x02, 0x00, b'i', b's', b'o',
        b'm', b'i', b's', b'o', b'2', b'm', b'p', b'4', b'1',
    ]);

    // moov
    mp4_buffer.extend_from_slice(&moov_box);

    // mdat
    let mdat_size = 8 + mdat_data.len();
    mp4_buffer.extend_from_slice(&(mdat_size as u32).to_be_bytes());
    mp4_buffer.extend_from_slice(b"mdat");
    mp4_buffer.extend_from_slice(&mdat_data);

    Ok(mp4_buffer)
}

fn build_moov(
    media_data: &MediaData,
    video_samples: &[Vec<u8>],
    audio_samples: &[Vec<u8>],
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
    video_data_end: usize,
) -> io::Result<Vec<u8>> {
    let mut moov = Vec::new();

    // Calculate global minimum PTS across all streams for proper synchronization
    let video_min_pts = media_data
        .frame_timestamps
        .iter()
        .filter_map(|(pts, _)| *pts)
        .min();

    let audio_min_pts = media_data
        .audio_timestamps
        .iter()
        .filter_map(|&pts| pts)
        .min();

    let global_min_pts = match (video_min_pts, audio_min_pts) {
        (Some(v), Some(a)) => v.min(a),
        (Some(v), None) => v,
        (None, Some(a)) => a,
        (None, None) => 0,
    };

    // mvhd
    let duration = video_samples.len() as u32 * 3000; // 90000 timescale, 30fps
    moov.extend_from_slice(&build_mvhd(duration, !audio_samples.is_empty()));

    // video trak
    moov.extend_from_slice(&build_video_trak(
        media_data,
        video_samples,
        &calculate_composition_offsets(&media_data.frame_timestamps, global_min_pts),
        ftyp_size,
        moov_size,
        mdat_header_size,
    )?);

    // audio trak (if present)
    if !audio_samples.is_empty() {
        moov.extend_from_slice(&build_audio_trak(
            media_data,
            audio_samples,
            global_min_pts,
            ftyp_size,
            moov_size,
            mdat_header_size,
            video_data_end,
        )?);
    }

    // Add moov header
    let total_size = 8 + moov.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"moov");
    result.extend_from_slice(&moov);

    Ok(result)
}

fn build_mvhd(duration: u32, has_audio: bool) -> Vec<u8> {
    let next_track_id = if has_audio { 3 } else { 2 };

    vec![
        0x00,
        0x00,
        0x00,
        0x6C, // size
        b'm',
        b'v',
        b'h',
        b'd',
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x00, // creation time
        0x00,
        0x00,
        0x00,
        0x00, // modification time
        0x00,
        0x01,
        0x5F,
        0x90, // timescale = 90000
        (duration >> 24) as u8,
        (duration >> 16) as u8,
        (duration >> 8) as u8,
        duration as u8,
        0x00,
        0x01,
        0x00,
        0x00, // rate
        0x01,
        0x00, // volume
        0x00,
        0x00, // reserved
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // reserved
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
        0x00, // matrix
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
        0x00, // pre-defined
        (next_track_id >> 24) as u8,
        (next_track_id >> 16) as u8,
        (next_track_id >> 8) as u8,
        next_track_id as u8, // next track ID
    ]
}

fn build_video_trak(
    media_data: &MediaData,
    samples: &[Vec<u8>],
    composition_offsets: &[i32],
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
) -> io::Result<Vec<u8>> {
    let mut trak = Vec::new();

    // tkhd
    trak.extend_from_slice(&build_tkhd(
        1,
        samples.len(),
        media_data.width,
        media_data.height,
    ));

    // mdia
    trak.extend_from_slice(&build_video_mdia(
        media_data,
        samples,
        composition_offsets,
        ftyp_size,
        moov_size,
        mdat_header_size,
    )?);

    let total_size = 8 + trak.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"trak");
    result.extend_from_slice(&trak);

    Ok(result)
}

fn build_tkhd(track_id: u32, sample_count: usize, width: u16, height: u16) -> Vec<u8> {
    let duration = sample_count as u32 * 3000;
    let width_fixed = (width as u32) << 16;
    let height_fixed = (height as u32) << 16;

    vec![
        0x00,
        0x00,
        0x00,
        0x5C, // size
        b't',
        b'k',
        b'h',
        b'd',
        0x00,
        0x00,
        0x00,
        0x07, // version + flags (enabled)
        0x00,
        0x00,
        0x00,
        0x00, // creation time
        0x00,
        0x00,
        0x00,
        0x00, // modification time
        (track_id >> 24) as u8,
        (track_id >> 16) as u8,
        (track_id >> 8) as u8,
        track_id as u8,
        0x00,
        0x00,
        0x00,
        0x00, // reserved
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
        0x00, // reserved
        0x00,
        0x00, // layer
        0x00,
        0x00, // alternate group
        0x00,
        0x00, // volume
        0x00,
        0x00, // reserved
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
        0x00, // matrix
        (width_fixed >> 24) as u8,
        (width_fixed >> 16) as u8,
        (width_fixed >> 8) as u8,
        width_fixed as u8,
        (height_fixed >> 24) as u8,
        (height_fixed >> 16) as u8,
        (height_fixed >> 8) as u8,
        height_fixed as u8,
    ]
}

fn build_video_mdia(
    media_data: &MediaData,
    samples: &[Vec<u8>],
    composition_offsets: &[i32],
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
) -> io::Result<Vec<u8>> {
    let mut mdia = Vec::new();

    // mdhd
    let duration = samples.len() as u32 * 3000;
    mdia.extend_from_slice(&[
        0x00,
        0x00,
        0x00,
        0x20, // size
        b'm',
        b'd',
        b'h',
        b'd',
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x00, // creation time
        0x00,
        0x00,
        0x00,
        0x00, // modification time
        0x00,
        0x01,
        0x5F,
        0x90, // timescale = 90000
        (duration >> 24) as u8,
        (duration >> 16) as u8,
        (duration >> 8) as u8,
        duration as u8,
        0x55,
        0xC4, // language
        0x00,
        0x00, // pre-defined
    ]);

    // hdlr
    mdia.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x21, // size
        b'h', b'd', b'l', b'r', 0x00, 0x00, 0x00, 0x00, // version + flags
        0x00, 0x00, 0x00, 0x00, // pre-defined
        b'v', b'i', b'd', b'e', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, // name
    ]);

    // minf
    mdia.extend_from_slice(&build_video_minf(
        media_data,
        samples,
        composition_offsets,
        ftyp_size,
        moov_size,
        mdat_header_size,
    )?);

    let total_size = 8 + mdia.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"mdia");
    result.extend_from_slice(&mdia);

    Ok(result)
}

fn build_video_minf(
    media_data: &MediaData,
    samples: &[Vec<u8>],
    composition_offsets: &[i32],
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
) -> io::Result<Vec<u8>> {
    let mut minf = Vec::new();

    // vmhd
    minf.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x14, // size = 20
        b'v', b'm', b'h', b'd', 0x00, 0x00, 0x00, 0x01, // version + flags
        0x00, 0x00, // graphics mode
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // opcolor (RGB)
    ]);

    // dinf
    minf.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x24, // size
        b'd', b'i', b'n', b'f', 0x00, 0x00, 0x00, 0x1C, // dref size
        b'd', b'r', b'e', b'f', 0x00, 0x00, 0x00, 0x00, // version + flags
        0x00, 0x00, 0x00, 0x01, // entry count
        0x00, 0x00, 0x00, 0x0C, // url size
        b'u', b'r', b'l', b' ', 0x00, 0x00, 0x00, 0x01, // version + flags (self-reference)
    ]);

    // stbl
    minf.extend_from_slice(&build_video_stbl(
        media_data,
        samples,
        composition_offsets,
        ftyp_size,
        moov_size,
        mdat_header_size,
    )?);

    let total_size = 8 + minf.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"minf");
    result.extend_from_slice(&minf);

    Ok(result)
}

fn build_video_stbl(
    media_data: &MediaData,
    samples: &[Vec<u8>],
    composition_offsets: &[i32],
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
) -> io::Result<Vec<u8>> {
    let mut stbl = Vec::new();

    // stsd
    stbl.extend_from_slice(&build_video_stsd(media_data)?);

    // stts
    let sample_count = samples.len() as u32;
    let mut stts = vec![
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x01, // entry count
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8,
        0x00,
        0x00,
        0x0B,
        0xB8, // sample delta = 3000
    ];
    let stts_size = 8 + stts.len();
    let mut stts_box = Vec::new();
    stts_box.extend_from_slice(&(stts_size as u32).to_be_bytes());
    stts_box.extend_from_slice(b"stts");
    stts_box.extend_from_slice(&stts);
    stbl.extend_from_slice(&stts_box);

    // stsc - Put all video samples in a single chunk
    stbl.extend_from_slice(&[
        0x00,
        0x00,
        0x00,
        0x1C, // size
        b's',
        b't',
        b's',
        b'c',
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x01, // entry count
        0x00,
        0x00,
        0x00,
        0x01, // first chunk
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8, // samples per chunk = all samples
        0x00,
        0x00,
        0x00,
        0x01, // sample description index
    ]);

    // stsz
    let mut stsz = vec![
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x00, // sample size (0 = variable)
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8,
    ];
    for sample in samples {
        let size = sample.len() as u32;
        stsz.extend_from_slice(&size.to_be_bytes());
    }
    let stsz_size = 8 + stsz.len();
    let mut stsz_box = Vec::new();
    stsz_box.extend_from_slice(&(stsz_size as u32).to_be_bytes());
    stsz_box.extend_from_slice(b"stsz");
    stsz_box.extend_from_slice(&stsz);
    stbl.extend_from_slice(&stsz_box);

    // stco - Only one chunk containing all video samples
    let base_offset = ftyp_size + moov_size + mdat_header_size;
    let chunk_count = 1u32;
    let mut stco = vec![
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        (chunk_count >> 24) as u8,
        (chunk_count >> 16) as u8,
        (chunk_count >> 8) as u8,
        chunk_count as u8,
    ];
    // Single chunk offset pointing to start of video data
    stco.extend_from_slice(&(base_offset as u32).to_be_bytes());
    let stco_size = 8 + stco.len();
    let mut stco_box = Vec::new();
    stco_box.extend_from_slice(&(stco_size as u32).to_be_bytes());
    stco_box.extend_from_slice(b"stco");
    stco_box.extend_from_slice(&stco);
    stbl.extend_from_slice(&stco_box);

    // ctts (composition time offsets)
    if !composition_offsets.is_empty() && composition_offsets.iter().any(|&o| o != 0) {
        let mut ctts = vec![
            0x00,
            0x00,
            0x00,
            0x00, // version + flags
            (sample_count >> 24) as u8,
            (sample_count >> 16) as u8,
            (sample_count >> 8) as u8,
            sample_count as u8,
        ];
        for &offset in composition_offsets {
            ctts.extend_from_slice(&1u32.to_be_bytes()); // sample count
            ctts.extend_from_slice(&(offset as u32).to_be_bytes()); // sample offset
        }
        let ctts_size = 8 + ctts.len();
        let mut ctts_box = Vec::new();
        ctts_box.extend_from_slice(&(ctts_size as u32).to_be_bytes());
        ctts_box.extend_from_slice(b"ctts");
        ctts_box.extend_from_slice(&ctts);
        stbl.extend_from_slice(&ctts_box);
    }

    let total_size = 8 + stbl.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"stbl");
    result.extend_from_slice(&stbl);

    Ok(result)
}

fn build_video_stsd(media_data: &MediaData) -> io::Result<Vec<u8>> {
    let mut stsd = vec![
        0x00, 0x00, 0x00, 0x00, // version + flags
        0x00, 0x00, 0x00, 0x01, // entry count
    ];

    // avc1 sample entry
    let mut avc1 = vec![
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // reserved
        0x00,
        0x01, // data reference index
        0x00,
        0x00, // pre-defined
        0x00,
        0x00, // reserved
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
        0x00, // pre-defined
        (media_data.width >> 8) as u8,
        (media_data.width & 0xFF) as u8,
        (media_data.height >> 8) as u8,
        (media_data.height & 0xFF) as u8,
        0x00,
        0x48,
        0x00,
        0x00, // horizontal resolution
        0x00,
        0x48,
        0x00,
        0x00, // vertical resolution
        0x00,
        0x00,
        0x00,
        0x00, // reserved
        0x00,
        0x01, // frame count
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
        0x00, // compressor name
        0x00,
        0x18, // depth
        0xFF,
        0xFF, // pre-defined
    ];

    // avcC
    if let (Some(sps), Some(pps)) = (&media_data.sps, &media_data.pps) {
        let mut avcc = vec![
            0x01, // configuration version
        ];

        if sps.len() >= 4 {
            avcc.push(sps[1]); // profile
            avcc.push(sps[2]); // profile compatibility
            avcc.push(sps[3]); // level
        } else {
            avcc.extend_from_slice(&[0x64, 0x00, 0x1F]);
        }

        avcc.push(0xFF); // 6 bits reserved + 2 bits NAL size length - 1
        avcc.push(0xE1); // 3 bits reserved + 5 bits number of SPS

        avcc.extend_from_slice(&(sps.len() as u16).to_be_bytes());
        avcc.extend_from_slice(sps);

        avcc.push(0x01); // number of PPS
        avcc.extend_from_slice(&(pps.len() as u16).to_be_bytes());
        avcc.extend_from_slice(pps);

        let avcc_size = 8 + avcc.len();
        let mut avcc_box = Vec::new();
        avcc_box.extend_from_slice(&(avcc_size as u32).to_be_bytes());
        avcc_box.extend_from_slice(b"avcC");
        avcc_box.extend_from_slice(&avcc);

        avc1.extend_from_slice(&avcc_box);
    }

    let avc1_size = 8 + avc1.len();
    let mut avc1_box = Vec::new();
    avc1_box.extend_from_slice(&(avc1_size as u32).to_be_bytes());
    avc1_box.extend_from_slice(b"avc1");
    avc1_box.extend_from_slice(&avc1);

    stsd.extend_from_slice(&avc1_box);

    let stsd_size = 8 + stsd.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(stsd_size as u32).to_be_bytes());
    result.extend_from_slice(b"stsd");
    result.extend_from_slice(&stsd);

    Ok(result)
}

fn build_audio_trak(
    media_data: &MediaData,
    samples: &[Vec<u8>],
    global_min_pts: u64,
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
    video_data_end: usize,
) -> io::Result<Vec<u8>> {
    let mut trak = Vec::new();

    // tkhd
    let duration = samples.len() as u32 * 1920; // Duration in movie timescale (90kHz): 1024 samples @ 48kHz = 1920 in 90kHz
    trak.extend_from_slice(&[
        0x00,
        0x00,
        0x00,
        0x5C, // size
        b't',
        b'k',
        b'h',
        b'd',
        0x00,
        0x00,
        0x00,
        0x07, // version + flags
        0x00,
        0x00,
        0x00,
        0x00, // creation time
        0x00,
        0x00,
        0x00,
        0x00, // modification time
        0x00,
        0x00,
        0x00,
        0x02, // track ID = 2
        0x00,
        0x00,
        0x00,
        0x00, // reserved
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
        0x00, // reserved
        0x00,
        0x00, // layer
        0x00,
        0x00, // alternate group
        0x01,
        0x00, // volume = 1.0
        0x00,
        0x00, // reserved
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
        0x00, // matrix
        0x00,
        0x00,
        0x00,
        0x00, // width
        0x00,
        0x00,
        0x00,
        0x00, // height
    ]);

    // mdia
    trak.extend_from_slice(&build_audio_mdia(
        media_data,
        samples,
        global_min_pts,
        ftyp_size,
        moov_size,
        mdat_header_size,
        video_data_end,
    )?);

    // Add Edit List if audio doesn't start at global minimum PTS
    if let Some(Some(first_audio_pts)) = media_data.audio_timestamps.first() {
        if *first_audio_pts > global_min_pts {
            // Calculate delay in 90kHz timeline
            let delay_90khz = *first_audio_pts - global_min_pts;
            // Convert to 48kHz for audio track
            let delay_48khz = ((delay_90khz as i64 * 48000) / 90000) as i32;

            // Create edit list with empty edit followed by media edit
            let media_duration_48khz = samples.len() as u32 * 1024;
            let segment_duration_90khz = media_duration_48khz as u64 * 90000 / 48000;

            let elst_size = 36u32; // 8 (box header) + 28 (elst content)
            let mut edts = vec![
                // elst box
                (elst_size >> 24) as u8,
                (elst_size >> 16) as u8,
                (elst_size >> 8) as u8,
                elst_size as u8,
                b'e',
                b'l',
                b's',
                b't',
                0x00,
                0x00,
                0x00,
                0x00, // version + flags
                0x00,
                0x00,
                0x00,
                0x01, // entry count = 1
                // Entry: segment duration (in movie timescale = 90000)
                ((segment_duration_90khz >> 24) as u8),
                ((segment_duration_90khz >> 16) as u8),
                ((segment_duration_90khz >> 8) as u8),
                (segment_duration_90khz as u8),
                // Media time: where to start in media timeline (in media timescale = 48000)
                (delay_48khz >> 24) as u8,
                (delay_48khz >> 16) as u8,
                (delay_48khz >> 8) as u8,
                delay_48khz as u8,
                0x00,
                0x01,
                0x00,
                0x00, // media rate = 1.0
            ];

            let edts_size = 8 + edts.len();
            let mut edts_box = Vec::new();
            edts_box.extend_from_slice(&(edts_size as u32).to_be_bytes());
            edts_box.extend_from_slice(b"edts");
            edts_box.extend_from_slice(&edts);

            trak.extend_from_slice(&edts_box);
        }
    }

    let total_size = 8 + trak.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"trak");
    result.extend_from_slice(&trak);

    Ok(result)
}

fn build_audio_mdia(
    media_data: &MediaData,
    samples: &[Vec<u8>],
    global_min_pts: u64,
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
    video_data_end: usize,
) -> io::Result<Vec<u8>> {
    let mut mdia = Vec::new();

    // mdhd
    let duration = samples.len() as u32 * 1920; // 1920 = 1024 samples @ 48kHz in 90kHz timebase
    mdia.extend_from_slice(&[
        0x00,
        0x00,
        0x00,
        0x20, // size
        b'm',
        b'd',
        b'h',
        b'd',
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x00, // creation time
        0x00,
        0x00,
        0x00,
        0x00, // modification time
        0x00,
        0x01,
        0x5F,
        0x90, // timescale = 90000 (same as video for consistency)
        (duration >> 24) as u8,
        (duration >> 16) as u8,
        (duration >> 8) as u8,
        duration as u8,
        0x55,
        0xC4, // language
        0x00,
        0x00, // pre-defined
    ]);

    // hdlr
    mdia.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x21, // size
        b'h', b'd', b'l', b'r', 0x00, 0x00, 0x00, 0x00, // version + flags
        0x00, 0x00, 0x00, 0x00, // pre-defined
        b's', b'o', b'u', b'n', 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, // name
    ]);

    // minf
    mdia.extend_from_slice(&build_audio_minf(
        media_data,
        samples,
        global_min_pts,
        ftyp_size,
        moov_size,
        mdat_header_size,
        video_data_end,
    )?);

    let total_size = 8 + mdia.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"mdia");
    result.extend_from_slice(&mdia);

    Ok(result)
}

fn build_audio_minf(
    media_data: &MediaData,
    samples: &[Vec<u8>],
    global_min_pts: u64,
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
    video_data_end: usize,
) -> io::Result<Vec<u8>> {
    let mut minf = Vec::new();

    // smhd
    minf.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x10, // size
        b's', b'm', b'h', b'd', 0x00, 0x00, 0x00, 0x00, // version + flags
        0x00, 0x00, // balance
        0x00, 0x00, // reserved
    ]);

    // dinf
    minf.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x24, // size
        b'd', b'i', b'n', b'f', 0x00, 0x00, 0x00, 0x1C, // dref size
        b'd', b'r', b'e', b'f', 0x00, 0x00, 0x00, 0x00, // version + flags
        0x00, 0x00, 0x00, 0x01, // entry count
        0x00, 0x00, 0x00, 0x0C, // url size
        b'u', b'r', b'l', b' ', 0x00, 0x00, 0x00, 0x01, // version + flags (self-reference)
    ]);

    // stbl
    minf.extend_from_slice(&build_audio_stbl(
        media_data,
        samples,
        global_min_pts,
        ftyp_size,
        moov_size,
        mdat_header_size,
        video_data_end,
    )?);

    let total_size = 8 + minf.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"minf");
    result.extend_from_slice(&minf);

    Ok(result)
}

fn build_audio_stbl(
    media_data: &MediaData,
    samples: &[Vec<u8>],
    global_min_pts: u64,
    ftyp_size: usize,
    moov_size: usize,
    mdat_header_size: usize,
    video_data_end: usize,
) -> io::Result<Vec<u8>> {
    let mut stbl = Vec::new();

    // stsd
    stbl.extend_from_slice(&build_audio_stsd()?);

    // stts
    let sample_count = samples.len() as u32;
    let mut stts = vec![
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x01, // entry count
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8,
        0x00,
        0x00,
        0x07,
        0x80, // sample delta = 1920 (1024 samples @ 48kHz = 0.021333s in 90kHz timebase)
    ];
    let stts_size = 8 + stts.len();
    let mut stts_box = Vec::new();
    stts_box.extend_from_slice(&(stts_size as u32).to_be_bytes());
    stts_box.extend_from_slice(b"stts");
    stts_box.extend_from_slice(&stts);
    stbl.extend_from_slice(&stts_box);

    // stsc - Put all audio samples in a single chunk for better compatibility
    stbl.extend_from_slice(&[
        0x00,
        0x00,
        0x00,
        0x1C, // size
        b's',
        b't',
        b's',
        b'c',
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x01, // entry count = 1
        0x00,
        0x00,
        0x00,
        0x01, // first chunk = 1
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8, // samples per chunk = all samples
        0x00,
        0x00,
        0x00,
        0x01, // sample description index
    ]);

    // stsz
    let mut stsz = vec![
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        0x00,
        0x00,
        0x00,
        0x00, // sample size (0 = variable)
        (sample_count >> 24) as u8,
        (sample_count >> 16) as u8,
        (sample_count >> 8) as u8,
        sample_count as u8,
    ];
    for sample in samples {
        let size = sample.len() as u32;
        stsz.extend_from_slice(&size.to_be_bytes());
    }
    let stsz_size = 8 + stsz.len();
    let mut stsz_box = Vec::new();
    stsz_box.extend_from_slice(&(stsz_size as u32).to_be_bytes());
    stsz_box.extend_from_slice(b"stsz");
    stsz_box.extend_from_slice(&stsz);
    stbl.extend_from_slice(&stsz_box);

    // stco - Only one chunk containing all audio samples
    let base_offset = ftyp_size + moov_size + mdat_header_size + video_data_end;
    let chunk_count = 1u32; // Single chunk containing all samples
    let mut stco = vec![
        0x00,
        0x00,
        0x00,
        0x00, // version + flags
        (chunk_count >> 24) as u8,
        (chunk_count >> 16) as u8,
        (chunk_count >> 8) as u8,
        chunk_count as u8,
    ];
    // Add the single chunk offset (start of audio data in mdat)
    stco.extend_from_slice(&(base_offset as u32).to_be_bytes());

    let stco_size = 8 + stco.len();
    let mut stco_box = Vec::new();
    stco_box.extend_from_slice(&(stco_size as u32).to_be_bytes());
    stco_box.extend_from_slice(b"stco");
    stco_box.extend_from_slice(&stco);
    stbl.extend_from_slice(&stco_box);

    let total_size = 8 + stbl.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(total_size as u32).to_be_bytes());
    result.extend_from_slice(b"stbl");
    result.extend_from_slice(&stbl);

    Ok(result)
}

fn build_audio_stsd() -> io::Result<Vec<u8>> {
    let mut stsd = vec![
        0x00, 0x00, 0x00, 0x00, // version + flags
        0x00, 0x00, 0x00, 0x01, // entry count
    ];

    // mp4a sample entry
    let mut mp4a = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // reserved
        0x00, 0x01, // data reference index
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // reserved (version 0)
        0x00, 0x02, // channel count = 2
        0x00, 0x10, // sample size = 16
        0x00, 0x00, // pre-defined
        0x00, 0x00, // reserved
        0xBB, 0x80, 0x00, 0x00, // sample rate = 48000 << 16
    ];

    // esds
    let mut esds_content = vec![
        0x00, 0x00, 0x00, 0x00, // version + flags
    ];

    // ES_Descriptor
    esds_content.push(0x03); // ES_DescrTag
    esds_content.push(0x19); // length
    esds_content.extend_from_slice(&[0x00, 0x01]); // ES_ID = 1
    esds_content.push(0x00); // flags

    // DecoderConfigDescriptor
    esds_content.push(0x04); // DecoderConfigDescrTag
    esds_content.push(0x11); // length
    esds_content.push(0x40); // object type = AAC
    esds_content.push(0x15); // stream type (Audio) + upstream flag
    esds_content.extend_from_slice(&[0x00, 0x03, 0x00]); // buffer size (768)
    esds_content.extend_from_slice(&[0x00, 0x01, 0xF4, 0x00]); // max bitrate
    esds_content.extend_from_slice(&[0x00, 0x01, 0xF4, 0x00]); // avg bitrate

    // DecoderSpecificInfo
    esds_content.push(0x05); // DecoderSpecificInfoTag
    esds_content.push(0x02); // length
                             // AudioSpecificConfig: AAC-LC (2), 48kHz (3), stereo (2)
                             // 5 bits: audioObjectType = 2 (AAC-LC)
                             // 4 bits: samplingFrequencyIndex = 3 (48000 Hz)
                             // 4 bits: channelConfiguration = 2 (stereo)
                             // 3 bits: frameLengthFlag, dependsOnCoreCoder, extensionFlag = 0
                             // Byte 1: 00010 011 = 0x13 (but should be 0001 0011 = 0x11)
                             // Wait, let me recalculate:
                             // audioObjectType = 2: 00010
                             // samplingFrequencyIndex = 3: 0011
                             // First byte = 00010 011 = 0x13
                             // Wait, that's wrong. Let's do it properly:
                             // First 5 bits: audioObjectType = 2 = 00010
                             // Next 4 bits: samplingFrequencyIndex = 3 = 0011
                             // Next 4 bits: channelConfiguration = 2 = 0010
                             // Next 3 bits: other flags = 000
                             // Total: 00010 0011 0010 000 = 0001 0001 1001 0000 = 0x11 0x90
    esds_content.extend_from_slice(&[0x11, 0x90]);

    // SLConfigDescriptor
    esds_content.push(0x06); // SLConfigDescrTag
    esds_content.push(0x01); // length
    esds_content.push(0x02); // predefined = 2

    let esds_size = 8 + esds_content.len();
    let mut esds_box = Vec::new();
    esds_box.extend_from_slice(&(esds_size as u32).to_be_bytes());
    esds_box.extend_from_slice(b"esds");
    esds_box.extend_from_slice(&esds_content);

    mp4a.extend_from_slice(&esds_box);

    let mp4a_size = 8 + mp4a.len();
    let mut mp4a_box = Vec::new();
    mp4a_box.extend_from_slice(&(mp4a_size as u32).to_be_bytes());
    mp4a_box.extend_from_slice(b"mp4a");
    mp4a_box.extend_from_slice(&mp4a);

    stsd.extend_from_slice(&mp4a_box);

    let stsd_size = 8 + stsd.len();
    let mut result = Vec::new();
    result.extend_from_slice(&(stsd_size as u32).to_be_bytes());
    result.extend_from_slice(b"stsd");
    result.extend_from_slice(&stsd);

    Ok(result)
}

fn split_into_frames(video_stream: &[u8]) -> Vec<Vec<u8>> {
    let mut frames = Vec::new();
    let mut current_frame = Vec::new();
    let mut i = 0;

    while i < video_stream.len() {
        // Check for start code
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

        // AUD (9) marks new frame - save previous frame
        if nal_type == 9 && !current_frame.is_empty() {
            frames.push(current_frame.clone());
            current_frame.clear();
        }

        // Find end of this NAL unit (next start code)
        let mut nal_end = nal_start + 1;
        let mut found_end = false;

        while nal_end + 3 < video_stream.len() {
            if video_stream[nal_end] == 0x00
                && video_stream[nal_end + 1] == 0x00
                && (video_stream[nal_end + 2] == 0x01
                    || (video_stream[nal_end + 2] == 0x00 && video_stream[nal_end + 3] == 0x01))
            {
                found_end = true;
                break;
            }
            nal_end += 1;
        }

        if !found_end {
            nal_end = video_stream.len();
        }

        // Add this NAL to current frame (with start code)
        current_frame.extend_from_slice(&video_stream[i..nal_end]);
        i = nal_end;
    }

    if !current_frame.is_empty() {
        frames.push(current_frame);
    }

    frames
}

fn convert_annexb_to_avcc(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Find start code
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

        let nal_type = data[nal_start] & 0x1F;

        // Find next start code to determine NAL unit end
        let mut nal_end = nal_start + 1;
        let mut found_end = false;

        while nal_end + 3 < data.len() {
            if data[nal_end] == 0x00
                && data[nal_end + 1] == 0x00
                && (data[nal_end + 2] == 0x01
                    || (data[nal_end + 2] == 0x00 && data[nal_end + 3] == 0x01))
            {
                found_end = true;
                break;
            }
            nal_end += 1;
        }

        if !found_end {
            nal_end = data.len();
        }

        // Skip SPS (7), PPS (8), and AUD (9) - these are stored elsewhere
        if nal_type != 7 && nal_type != 8 && nal_type != 9 {
            let nal_size = nal_end - nal_start;
            if nal_size > 0 {
                // Write NAL size (4 bytes, big-endian)
                output.extend_from_slice(&(nal_size as u32).to_be_bytes());
                // Write NAL data (without start code)
                output.extend_from_slice(&data[nal_start..nal_end]);
            }
        }

        i = nal_end;
    }

    output
}

fn calculate_composition_offsets(
    timestamps: &[(Option<u64>, Option<u64>)],
    global_min_pts: u64,
) -> Vec<i32> {
    if timestamps.is_empty() {
        return Vec::new();
    }

    // Use global minimum PTS for synchronization across all streams
    let min_dts = global_min_pts;

    let mut offsets = Vec::new();

    for (pts, dts) in timestamps {
        let offset = match (pts, dts) {
            (Some(p), Some(d)) => {
                let normalized_pts = (*p as i64 - min_dts as i64) as i32;
                let normalized_dts = (*d as i64 - min_dts as i64) as i32;
                normalized_pts - normalized_dts
            }
            _ => 0,
        };
        offsets.push(offset);
    }

    if let Some(&first_offset) = offsets.first() {
        let adjustment = first_offset;
        for offset in &mut offsets {
            *offset -= adjustment;
        }
    }

    offsets
}
