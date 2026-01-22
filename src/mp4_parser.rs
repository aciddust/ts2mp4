#![allow(dead_code)]

use std::io::{self, ErrorKind};

/// MP4 박스 (Box/Atom) 구조
#[derive(Debug, Clone)]
pub struct Mp4Box {
    pub box_type: [u8; 4],
    pub size: u64,
    pub offset: usize,
    pub data: Vec<u8>,
}

/// mdat 박스 정보 (큰 데이터는 복사하지 않음)
#[derive(Debug)]
pub struct MdatBox {
    pub offset: usize,
    pub size: u64,
    pub data_offset: usize,
    pub data_size: usize,
}

/// 파싱된 MP4 파일 정보
#[derive(Debug)]
pub struct Mp4File {
    pub ftyp: Option<Mp4Box>,
    pub moov: Option<Mp4Box>,
    pub mdat: Option<MdatBox>,
    pub boxes: Vec<Mp4Box>,
    pub all_boxes_in_order: Vec<BoxInfo>, // 모든 박스의 순서 유지
}

/// 박스 정보 (타입별로 다르게 저장)
#[derive(Debug)]
pub enum BoxInfo {
    Small(Mp4Box), // ftyp, moov, styp, moof, emsg 등
    Mdat(MdatBox), // 큰 mdat는 오프셋만
}

/// Sample 정보 (fragment용)
#[derive(Debug, Clone)]
struct FragmentSampleInfo {
    duration: u32,
    size: u32,
    flags: u32,
    composition_time_offset: i32,
}

/// Track의 Fragment 데이터
#[derive(Debug)]
struct TrackFragments {
    track_id: u32,
    samples: Vec<FragmentSampleInfo>,
    mdat_data: Vec<u8>, // 모든 mdat 데이터를 순서대로
    timescale: u32,
    codec_info: Vec<u8>, // stsd (sample description)
}

/// MP4 바이트 읽기 헬퍼
struct Mp4Reader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Mp4Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Mp4Reader { data, offset: 0 }
    }

    fn read_u32(&mut self) -> io::Result<u32> {
        if self.offset + 4 > self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "Not enough data"));
        }
        let value = u32::from_be_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
        ]);
        self.offset += 4;
        Ok(value)
    }

    fn read_u64(&mut self) -> io::Result<u64> {
        if self.offset + 8 > self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "Not enough data"));
        }
        let value = u64::from_be_bytes([
            self.data[self.offset],
            self.data[self.offset + 1],
            self.data[self.offset + 2],
            self.data[self.offset + 3],
            self.data[self.offset + 4],
            self.data[self.offset + 5],
            self.data[self.offset + 6],
            self.data[self.offset + 7],
        ]);
        self.offset += 8;
        Ok(value)
    }

    fn read_bytes(&mut self, len: usize) -> io::Result<&'a [u8]> {
        if self.offset + len > self.data.len() {
            return Err(io::Error::new(ErrorKind::UnexpectedEof, "Not enough data"));
        }
        let bytes = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Ok(bytes)
    }

    fn seek(&mut self, offset: usize) {
        self.offset = offset;
    }

    fn position(&self) -> usize {
        self.offset
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }
}

/// MP4 파일 파싱
pub fn parse_mp4(data: &[u8]) -> io::Result<Mp4File> {
    let mut reader = Mp4Reader::new(data);
    let mut boxes = Vec::new();
    let mut all_boxes_in_order = Vec::new();
    let mut ftyp = None;
    let mut moov = None;
    let mut mdat = None; // 첫 번째 mdat만 (호환성용)

    while reader.remaining() >= 8 {
        let box_start = reader.position();

        // 박스 크기 읽기
        let size32 = reader.read_u32()?;
        let box_type_bytes = reader.read_bytes(4)?;
        let mut box_type = [0u8; 4];
        box_type.copy_from_slice(box_type_bytes);

        // 실제 박스 크기 계산
        let box_size = if size32 == 1 {
            // 64-bit size
            reader.read_u64()?
        } else if size32 == 0 {
            // 파일 끝까지
            (data.len() - box_start) as u64
        } else {
            size32 as u64
        };

        // 박스 데이터 계산
        let header_size = if size32 == 1 { 16 } else { 8 };
        let data_size = box_size.saturating_sub(header_size);

        if data_size > (data.len() - reader.position()) as u64 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Box size {} exceeds available data", box_size),
            ));
        }

        // mdat는 큰 데이터이므로 복사하지 않고 오프셋만 기록
        if &box_type == b"mdat" {
            let mdat_box = MdatBox {
                offset: box_start,
                size: box_size,
                data_offset: reader.position(),
                data_size: data_size as usize,
            };

            // 첫 번째 mdat 저장 (호환성)
            if mdat.is_none() {
                mdat = Some(MdatBox {
                    offset: box_start,
                    size: box_size,
                    data_offset: reader.position(),
                    data_size: data_size as usize,
                });
            }

            all_boxes_in_order.push(BoxInfo::Mdat(mdat_box));

            // mdat 데이터 건너뛰기
            reader.seek(reader.position() + data_size as usize);
            continue;
        }

        let box_data = reader.read_bytes(data_size as usize)?.to_vec();

        let mp4_box = Mp4Box {
            box_type,
            size: box_size,
            offset: box_start,
            data: box_data,
        };

        // 주요 박스 저장
        match &box_type {
            b"ftyp" => ftyp = Some(mp4_box.clone()),
            b"moov" => moov = Some(mp4_box.clone()),
            _ => {}
        }

        all_boxes_in_order.push(BoxInfo::Small(mp4_box.clone()));
        boxes.push(mp4_box);
    }

    Ok(Mp4File {
        ftyp,
        moov,
        mdat,
        boxes,
        all_boxes_in_order,
    })
}

/// moov 박스 내부의 자식 박스들을 파싱
pub fn parse_container_box(data: &[u8]) -> io::Result<Vec<Mp4Box>> {
    let mut reader = Mp4Reader::new(data);
    let mut boxes = Vec::new();

    while reader.remaining() >= 8 {
        let box_start = reader.position();

        let size32 = reader.read_u32()?;
        let box_type_bytes = reader.read_bytes(4)?;
        let mut box_type = [0u8; 4];
        box_type.copy_from_slice(box_type_bytes);

        let box_size = if size32 == 1 {
            reader.read_u64()?
        } else if size32 == 0 {
            (data.len() - box_start) as u64
        } else {
            size32 as u64
        };

        let header_size = if size32 == 1 { 16 } else { 8 };
        let data_size = box_size.saturating_sub(header_size);

        if data_size > reader.remaining() as u64 {
            break;
        }

        let box_data = reader.read_bytes(data_size as usize)?.to_vec();

        boxes.push(Mp4Box {
            box_type,
            size: box_size,
            offset: box_start,
            data: box_data,
        });
    }

    Ok(boxes)
}

/// 박스를 이름으로 찾기
pub fn find_box<'a>(boxes: &'a [Mp4Box], box_type: &[u8; 4]) -> Option<&'a Mp4Box> {
    boxes.iter().find(|b| &b.box_type == box_type)
}

/// 박스 경로로 찾기 (예: moov/trak/mdia/minf)
pub fn find_box_path(data: &[u8], path: &[&[u8; 4]]) -> io::Result<Option<Mp4Box>> {
    if path.is_empty() {
        return Ok(None);
    }

    let boxes = parse_container_box(data)?;

    if let Some(first_box) = find_box(&boxes, path[0]) {
        if path.len() == 1 {
            return Ok(Some(first_box.clone()));
        }
        // 재귀적으로 하위 박스 탐색
        return find_box_path(&first_box.data, &path[1..]);
    }

    Ok(None)
}

/// FullBox의 version과 flags 읽기
pub fn read_full_box_header(data: &[u8]) -> io::Result<(u8, u32)> {
    if data.len() < 4 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "Not enough data for FullBox header",
        ));
    }

    let version = data[0];
    let flags = u32::from_be_bytes([0, data[1], data[2], data[3]]);

    Ok((version, flags))
}

/// 타임스탬프 정보
#[derive(Debug, Clone)]
pub struct TimestampInfo {
    pub timescale: u32,
    pub duration: u64,
    pub creation_time: u64,
    pub modification_time: u64,
}

/// mvhd (Movie Header Box) 파싱
pub fn parse_mvhd(data: &[u8]) -> io::Result<TimestampInfo> {
    let mut reader = Mp4Reader::new(data);
    let (version, _flags) = read_full_box_header(data)?;
    reader.seek(4); // Skip version+flags

    let (creation_time, modification_time, timescale, duration) = if version == 1 {
        // 64-bit version
        let creation = reader.read_u64()?;
        let modification = reader.read_u64()?;
        let ts = reader.read_u32()?;
        let dur = reader.read_u64()?;
        (creation, modification, ts, dur)
    } else {
        // 32-bit version
        let creation = reader.read_u32()? as u64;
        let modification = reader.read_u32()? as u64;
        let ts = reader.read_u32()?;
        let dur = reader.read_u32()? as u64;
        (creation, modification, ts, dur)
    };

    Ok(TimestampInfo {
        timescale,
        duration,
        creation_time,
        modification_time,
    })
}

/// mdhd (Media Header Box) 파싱
pub fn parse_mdhd(data: &[u8]) -> io::Result<TimestampInfo> {
    let mut reader = Mp4Reader::new(data);
    let (version, _flags) = read_full_box_header(data)?;
    reader.seek(4);

    let (creation_time, modification_time, timescale, duration) = if version == 1 {
        let creation = reader.read_u64()?;
        let modification = reader.read_u64()?;
        let ts = reader.read_u32()?;
        let dur = reader.read_u64()?;
        (creation, modification, ts, dur)
    } else {
        let creation = reader.read_u32()? as u64;
        let modification = reader.read_u32()? as u64;
        let ts = reader.read_u32()?;
        let dur = reader.read_u32()? as u64;
        (creation, modification, ts, dur)
    };

    Ok(TimestampInfo {
        timescale,
        duration,
        creation_time,
        modification_time,
    })
}

/// stts (Decoding Time to Sample Box) 엔트리
#[derive(Debug, Clone)]
pub struct SttsEntry {
    pub sample_count: u32,
    pub sample_delta: u32,
}

/// stts 박스 파싱
pub fn parse_stts(data: &[u8]) -> io::Result<Vec<SttsEntry>> {
    let mut reader = Mp4Reader::new(data);
    let (_version, _flags) = read_full_box_header(data)?;
    reader.seek(4);

    let entry_count = reader.read_u32()?;
    let mut entries = Vec::with_capacity(entry_count as usize);

    for _ in 0..entry_count {
        entries.push(SttsEntry {
            sample_count: reader.read_u32()?,
            sample_delta: reader.read_u32()?,
        });
    }

    Ok(entries)
}

/// ctts (Composition Time to Sample Box) 엔트리
#[derive(Debug, Clone)]
pub struct CttsEntry {
    pub sample_count: u32,
    pub sample_offset: i32,
}

/// ctts 박스 파싱
pub fn parse_ctts(data: &[u8]) -> io::Result<Vec<CttsEntry>> {
    let mut reader = Mp4Reader::new(data);
    let (version, _flags) = read_full_box_header(data)?;
    reader.seek(4);

    let entry_count = reader.read_u32()?;
    let mut entries = Vec::with_capacity(entry_count as usize);

    for _ in 0..entry_count {
        let sample_count = reader.read_u32()?;
        let sample_offset = if version == 0 {
            reader.read_u32()? as i32
        } else {
            // version 1: signed offset
            let offset_u32 = reader.read_u32()?;
            offset_u32 as i32
        };

        entries.push(CttsEntry {
            sample_count,
            sample_offset,
        });
    }

    Ok(entries)
}

/// 모든 트랙의 타임스탬프 정보 추출
pub fn extract_all_timestamps(mp4: &Mp4File) -> io::Result<Vec<TimestampInfo>> {
    let mut timestamps = Vec::new();

    if let Some(moov) = &mp4.moov {
        let moov_boxes = parse_container_box(&moov.data)?;

        // mvhd에서 전역 타임스탬프
        if let Some(mvhd_box) = find_box(&moov_boxes, b"mvhd") {
            timestamps.push(parse_mvhd(&mvhd_box.data)?);
        }

        // 각 트랙의 mdhd
        for trak_box in moov_boxes.iter().filter(|b| &b.box_type == b"trak") {
            if let Ok(Some(mdhd_box)) = find_box_path(&trak_box.data, &[b"mdia", b"mdhd"]) {
                timestamps.push(parse_mdhd(&mdhd_box.data)?);
            }
        }
    }

    Ok(timestamps)
}

/// 샘플 정보
#[derive(Debug, Clone)]
pub struct SampleInfo {
    pub size: u32,
    pub offset: u64,
    pub duration: u32,
    pub composition_offset: i32,
}

/// stsz (Sample Size Box) 파싱
pub fn parse_stsz(data: &[u8]) -> io::Result<Vec<u32>> {
    let mut reader = Mp4Reader::new(data);
    let (_version, _flags) = read_full_box_header(data)?;
    reader.seek(4);

    let sample_size = reader.read_u32()?;
    let sample_count = reader.read_u32()?;

    if sample_size != 0 {
        // 모든 샘플이 같은 크기
        Ok(vec![sample_size; sample_count as usize])
    } else {
        // 각 샘플의 크기가 다름
        let mut sizes = Vec::with_capacity(sample_count as usize);
        for _ in 0..sample_count {
            sizes.push(reader.read_u32()?);
        }
        Ok(sizes)
    }
}

/// stco (Chunk Offset Box) 파싱 - 32-bit
pub fn parse_stco(data: &[u8]) -> io::Result<Vec<u64>> {
    let mut reader = Mp4Reader::new(data);
    let (_version, _flags) = read_full_box_header(data)?;
    reader.seek(4);

    let entry_count = reader.read_u32()?;
    let mut offsets = Vec::with_capacity(entry_count as usize);

    for _ in 0..entry_count {
        offsets.push(reader.read_u32()? as u64);
    }

    Ok(offsets)
}

/// co64 (Chunk Offset Box) 파싱 - 64-bit
pub fn parse_co64(data: &[u8]) -> io::Result<Vec<u64>> {
    let mut reader = Mp4Reader::new(data);
    let (_version, _flags) = read_full_box_header(data)?;
    reader.seek(4);

    let entry_count = reader.read_u32()?;
    let mut offsets = Vec::with_capacity(entry_count as usize);

    for _ in 0..entry_count {
        offsets.push(reader.read_u64()?);
    }

    Ok(offsets)
}

/// stsc (Sample to Chunk Box) 엔트리
#[derive(Debug, Clone)]
pub struct StscEntry {
    pub first_chunk: u32,
    pub samples_per_chunk: u32,
    pub sample_description_index: u32,
}

/// stsc 박스 파싱
pub fn parse_stsc(data: &[u8]) -> io::Result<Vec<StscEntry>> {
    let mut reader = Mp4Reader::new(data);
    let (_version, _flags) = read_full_box_header(data)?;
    reader.seek(4);

    let entry_count = reader.read_u32()?;
    let mut entries = Vec::with_capacity(entry_count as usize);

    for _ in 0..entry_count {
        entries.push(StscEntry {
            first_chunk: reader.read_u32()?,
            samples_per_chunk: reader.read_u32()?,
            sample_description_index: reader.read_u32()?,
        });
    }

    Ok(entries)
}

/// 트랙 정보
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub track_id: u32,
    pub timescale: u32,
    pub duration: u64,
    pub media_type: MediaType,
    pub samples: Vec<SampleInfo>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Video,
    Audio,
    Unknown,
}

/// hdlr (Handler Reference Box)에서 미디어 타입 파싱
pub fn parse_hdlr(data: &[u8]) -> io::Result<MediaType> {
    let mut reader = Mp4Reader::new(data);
    let (_version, _flags) = read_full_box_header(data)?;
    reader.seek(4);

    // pre_defined (4 bytes)
    reader.read_u32()?;

    // handler_type (4 bytes)
    let handler_type = reader.read_bytes(4)?;

    match handler_type {
        b"vide" => Ok(MediaType::Video),
        b"soun" => Ok(MediaType::Audio),
        _ => Ok(MediaType::Unknown),
    }
}

/// 트랙 정보 추출
pub fn extract_track_info(trak_data: &[u8]) -> io::Result<TrackInfo> {
    let trak_boxes = parse_container_box(trak_data)?;

    // tkhd에서 track_id
    let track_id = if let Some(tkhd_box) = find_box(&trak_boxes, b"tkhd") {
        let mut reader = Mp4Reader::new(&tkhd_box.data);
        let (version, _flags) = read_full_box_header(&tkhd_box.data)?;
        reader.seek(4);

        if version == 1 {
            reader.read_u64()?; // creation_time
            reader.read_u64()?; // modification_time
            reader.read_u32()? // track_id
        } else {
            reader.read_u32()?; // creation_time
            reader.read_u32()?; // modification_time
            reader.read_u32()? // track_id
        }
    } else {
        0
    };

    // mdia에서 mdhd와 hdlr
    let mdia_box = find_box(&trak_boxes, b"mdia")
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "mdia box not found"))?;

    let mdia_boxes = parse_container_box(&mdia_box.data)?;

    let timestamp_info = if let Some(mdhd_box) = find_box(&mdia_boxes, b"mdhd") {
        parse_mdhd(&mdhd_box.data)?
    } else {
        return Err(io::Error::new(ErrorKind::InvalidData, "mdhd box not found"));
    };

    let media_type = if let Some(hdlr_box) = find_box(&mdia_boxes, b"hdlr") {
        parse_hdlr(&hdlr_box.data)?
    } else {
        MediaType::Unknown
    };

    Ok(TrackInfo {
        track_id,
        timescale: timestamp_info.timescale,
        duration: timestamp_info.duration,
        media_type,
        samples: Vec::new(), // 샘플은 별도로 추출
    })
}

/// 박스를 출력 버퍼에 쓰기 (크기 자동 처리)
fn write_box(output: &mut Vec<u8>, box_type: &[u8; 4], data: &[u8]) {
    let total_size = 8 + data.len();

    if total_size <= u32::MAX as usize {
        // 32-bit size
        output.extend_from_slice(&(total_size as u32).to_be_bytes());
        output.extend_from_slice(box_type);
        output.extend_from_slice(data);
    } else {
        // 64-bit size
        output.extend_from_slice(&1u32.to_be_bytes()); // size = 1
        output.extend_from_slice(box_type);
        output.extend_from_slice(&(total_size as u64).to_be_bytes());
        output.extend_from_slice(data);
    }
}

/// MP4 파일에서 타임스탬프를 리셋하여 새로운 MP4 생성
pub fn reset_mp4_timestamps(data: &[u8]) -> io::Result<Vec<u8>> {
    let mp4 = parse_mp4(data)?;

    // Fragmented MP4인 경우: 첫 번째 tfdt 값을 찾아서 offset으로 사용
    let first_tfdt_offsets = extract_first_tfdt_values(&mp4.all_boxes_in_order)?;

    // Fragmented MP4의 총 duration 계산
    let total_durations =
        calculate_fragment_durations(&mp4.all_boxes_in_order, &first_tfdt_offsets)?;

    // moov 박스 수정 (total duration 전달)
    let new_moov = if let Some(moov) = &mp4.moov {
        reset_moov_timestamps_with_duration(&moov.data, &total_durations)?
    } else {
        return Err(io::Error::new(ErrorKind::InvalidData, "moov box not found"));
    };

    // 새 MP4 파일 구성 - 원본 순서대로
    let mut output = Vec::new();
    let mut moov_written = false;

    for box_info in &mp4.all_boxes_in_order {
        match box_info {
            BoxInfo::Small(mp4_box) => {
                match &mp4_box.box_type {
                    b"ftyp" => {
                        write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
                    }
                    b"moov" => {
                        // moov는 리셋된 버전 사용
                        if !moov_written {
                            write_box(&mut output, b"moov", &new_moov);
                            moov_written = true;
                        }
                    }
                    b"moof" => {
                        // Fragmented MP4의 fragment 타임스탬프를 상대적으로 조정
                        let new_moof =
                            reset_moof_timestamps_relative(&mp4_box.data, &first_tfdt_offsets)?;
                        write_box(&mut output, b"moof", &new_moof);
                    }
                    _ => {
                        // styp, emsg 등은 그대로 복사
                        write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
                    }
                }
            }
            BoxInfo::Mdat(mdat) => {
                // mdat는 원본 데이터에서 직접 복사
                let mdat_data = &data[mdat.data_offset..mdat.data_offset + mdat.data_size];
                write_box(&mut output, b"mdat", mdat_data);
            }
        }
    }

    Ok(output)
}

/// moov 박스의 타임스탬프 리셋 (duration 포함)
fn reset_moov_timestamps_with_duration(
    moov_data: &[u8],
    track_durations: &[u64],
) -> io::Result<Vec<u8>> {
    let moov_boxes = parse_container_box(moov_data)?;
    let mut output = Vec::new();

    // 비디오 트랙(track 0)의 duration을 movie duration으로 사용
    // (각 트랙의 timescale이 다르므로 단순 max는 부적절)
    let movie_duration = track_durations.first().copied().unwrap_or(0);
    let mut track_index = 0;

    for mp4_box in &moov_boxes {
        match &mp4_box.box_type {
            b"mvhd" => {
                // mvhd 타임스탬프 리셋 및 duration 설정
                let new_mvhd = reset_mvhd_with_duration(&mp4_box.data, movie_duration)?;
                write_box(&mut output, b"mvhd", &new_mvhd);
            }
            b"trak" => {
                // trak 내부 재귀 처리 (해당 트랙의 duration 전달)
                let trak_duration = if track_index < track_durations.len() {
                    track_durations[track_index]
                } else {
                    0
                };
                let new_trak = reset_trak_timestamps_with_duration(&mp4_box.data, trak_duration)?;
                write_box(&mut output, b"trak", &new_trak);
                track_index += 1;
            }
            _ => {
                // 다른 박스는 그대로 복사
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// mvhd 박스 타임스탬프 리셋 및 duration 설정
fn reset_mvhd_with_duration(data: &[u8], duration: u64) -> io::Result<Vec<u8>> {
    let mut output = data.to_vec();
    let (version, _) = read_full_box_header(data)?;

    if version == 1 {
        // 64-bit version
        output[4..12].fill(0); // creation_time = 0
        output[12..20].fill(0); // modification_time = 0

        // timescale at offset 20 (4 bytes)
        let timescale = u32::from_be_bytes([output[20], output[21], output[22], output[23]]);

        // duration at offset 24 (8 bytes) - 이미 해당 timescale 기준
        output[24..32].copy_from_slice(&duration.to_be_bytes());
    } else {
        // 32-bit version
        output[4..8].fill(0); // creation_time = 0
        output[8..12].fill(0); // modification_time = 0

        // timescale at offset 12 (4 bytes)
        let timescale = u32::from_be_bytes([output[12], output[13], output[14], output[15]]);

        // duration at offset 16 (4 bytes)
        let duration_32 = if duration > u32::MAX as u64 {
            u32::MAX
        } else {
            duration as u32
        };
        output[16..20].copy_from_slice(&duration_32.to_be_bytes());
    }

    Ok(output)
}

/// trak 박스 타임스탬프 리셋 및 duration 설정
fn reset_trak_timestamps_with_duration(trak_data: &[u8], duration: u64) -> io::Result<Vec<u8>> {
    let trak_boxes = parse_container_box(trak_data)?;
    let mut output = Vec::new();

    for mp4_box in &trak_boxes {
        match &mp4_box.box_type {
            b"tkhd" => {
                let new_tkhd = reset_tkhd_with_duration(&mp4_box.data, duration)?;
                write_box(&mut output, b"tkhd", &new_tkhd);
            }
            b"mdia" => {
                let new_mdia = reset_mdia_timestamps_with_duration(&mp4_box.data, duration)?;
                write_box(&mut output, b"mdia", &new_mdia);
            }
            _ => {
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// tkhd 박스 타임스탬프 리셋 및 duration 설정
fn reset_tkhd_with_duration(data: &[u8], duration: u64) -> io::Result<Vec<u8>> {
    let mut output = data.to_vec();
    let (version, _) = read_full_box_header(data)?;

    if version == 1 {
        output[4..12].fill(0); // creation_time = 0
        output[12..20].fill(0); // modification_time = 0
                                // duration at offset 24 (after track_id and reserved)
        output[24..32].copy_from_slice(&duration.to_be_bytes());
    } else {
        output[4..8].fill(0); // creation_time = 0
        output[8..12].fill(0); // modification_time = 0
                               // duration at offset 16
        let duration_32 = if duration > u32::MAX as u64 {
            u32::MAX
        } else {
            duration as u32
        };
        output[16..20].copy_from_slice(&duration_32.to_be_bytes());
    }

    Ok(output)
}

/// mdia 박스 타임스탬프 리셋 및 duration 설정
fn reset_mdia_timestamps_with_duration(mdia_data: &[u8], duration: u64) -> io::Result<Vec<u8>> {
    let mdia_boxes = parse_container_box(mdia_data)?;
    let mut output = Vec::new();

    for mp4_box in &mdia_boxes {
        match &mp4_box.box_type {
            b"mdhd" => {
                let new_mdhd = reset_mdhd_with_duration(&mp4_box.data, duration)?;
                write_box(&mut output, b"mdhd", &new_mdhd);
            }
            _ => {
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// mdhd 박스 타임스탬프 리셋 및 duration 설정
fn reset_mdhd_with_duration(data: &[u8], duration: u64) -> io::Result<Vec<u8>> {
    let mut output = data.to_vec();
    let (version, _) = read_full_box_header(data)?;

    if version == 1 {
        output[4..12].fill(0); // creation_time = 0
        output[12..20].fill(0); // modification_time = 0
                                // duration at offset 24 (after timescale)
        output[24..32].copy_from_slice(&duration.to_be_bytes());
    } else {
        output[4..8].fill(0); // creation_time = 0
        output[8..12].fill(0); // modification_time = 0
                               // duration at offset 16
        let duration_32 = if duration > u32::MAX as u64 {
            u32::MAX
        } else {
            duration as u32
        };
        output[16..20].copy_from_slice(&duration_32.to_be_bytes());
    }

    Ok(output)
}

/// moov 박스의 타임스탬프 리셋
fn reset_moov_timestamps(moov_data: &[u8]) -> io::Result<Vec<u8>> {
    let moov_boxes = parse_container_box(moov_data)?;
    let mut output = Vec::new();

    for mp4_box in &moov_boxes {
        match &mp4_box.box_type {
            b"mvhd" => {
                // mvhd 타임스탬프 리셋
                let new_mvhd = reset_mvhd(&mp4_box.data)?;
                write_box(&mut output, b"mvhd", &new_mvhd);
            }
            b"trak" => {
                // trak 내부 재귀 처리
                let new_trak = reset_trak_timestamps(&mp4_box.data)?;
                write_box(&mut output, b"trak", &new_trak);
            }
            _ => {
                // 다른 박스는 그대로 복사
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// mvhd 박스 타임스탬프 리셋
fn reset_mvhd(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut output = data.to_vec();
    let (version, _) = read_full_box_header(data)?;

    if version == 1 {
        // 64-bit version: creation_time at offset 4, modification_time at 12
        output[4..12].fill(0); // creation_time = 0
        output[12..20].fill(0); // modification_time = 0
    } else {
        // 32-bit version: creation_time at offset 4, modification_time at 8
        output[4..8].fill(0); // creation_time = 0
        output[8..12].fill(0); // modification_time = 0
    }

    Ok(output)
}

/// trak 박스 타임스탬프 리셋
fn reset_trak_timestamps(trak_data: &[u8]) -> io::Result<Vec<u8>> {
    let trak_boxes = parse_container_box(trak_data)?;
    let mut output = Vec::new();

    for mp4_box in &trak_boxes {
        match &mp4_box.box_type {
            b"tkhd" => {
                let new_tkhd = reset_tkhd(&mp4_box.data)?;
                output.extend_from_slice(&((8 + new_tkhd.len()) as u32).to_be_bytes());
                output.extend_from_slice(b"tkhd");
                output.extend_from_slice(&new_tkhd);
            }
            b"mdia" => {
                let new_mdia = reset_mdia_timestamps(&mp4_box.data)?;
                output.extend_from_slice(&((8 + new_mdia.len()) as u32).to_be_bytes());
                output.extend_from_slice(b"mdia");
                output.extend_from_slice(&new_mdia);
            }
            b"edts" => {
                // Edit list 제거 (타임스탬프가 0부터 시작하므로 불필요)
                // 건너뜀
                continue;
            }
            _ => {
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// tkhd 박스 타임스탬프 리셋
fn reset_tkhd(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut output = data.to_vec();
    let (version, _) = read_full_box_header(data)?;

    if version == 1 {
        output[4..12].fill(0); // creation_time
        output[12..20].fill(0); // modification_time
    } else {
        output[4..8].fill(0); // creation_time
        output[8..12].fill(0); // modification_time
    }

    Ok(output)
}

/// mdia 박스 타임스탬프 리셋
fn reset_mdia_timestamps(mdia_data: &[u8]) -> io::Result<Vec<u8>> {
    let mdia_boxes = parse_container_box(mdia_data)?;
    let mut output = Vec::new();

    for mp4_box in &mdia_boxes {
        match &mp4_box.box_type {
            b"mdhd" => {
                let new_mdhd = reset_mdhd(&mp4_box.data)?;
                write_box(&mut output, b"mdhd", &new_mdhd);
            }
            _ => {
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// mdhd 박스 타임스탬프 리셋
fn reset_mdhd(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut output = data.to_vec();
    let (version, _) = read_full_box_header(data)?;

    if version == 1 {
        output[4..12].fill(0); // creation_time
        output[12..20].fill(0); // modification_time
    } else {
        output[4..8].fill(0); // creation_time
        output[8..12].fill(0); // modification_time
    }

    Ok(output)
}

/// moof 박스 타임스탬프 리셋 (Fragmented MP4)
fn reset_moof_timestamps(moof_data: &[u8]) -> io::Result<Vec<u8>> {
    let moof_boxes = parse_container_box(moof_data)?;
    let mut output = Vec::new();

    for mp4_box in &moof_boxes {
        match &mp4_box.box_type {
            b"traf" => {
                // traf (Track Fragment) 안의 tfdt 리셋
                let new_traf = reset_traf_timestamps(&mp4_box.data)?;
                write_box(&mut output, b"traf", &new_traf);
            }
            _ => {
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// 모든 moof에서 첫 번째 tfdt 값 추출 (트랙별)
fn extract_first_tfdt_values(boxes: &[BoxInfo]) -> io::Result<Vec<u64>> {
    let mut first_tfdts: Vec<Option<u64>> = Vec::new();

    for box_info in boxes {
        if let BoxInfo::Small(mp4_box) = box_info {
            if &mp4_box.box_type == b"moof" {
                let moof_boxes = parse_container_box(&mp4_box.data)?;

                for moof_child in &moof_boxes {
                    if &moof_child.box_type == b"traf" {
                        let traf_boxes = parse_container_box(&moof_child.data)?;

                        // track_id 찾기
                        let mut track_index = 0;
                        for traf_child in &traf_boxes {
                            if &traf_child.box_type == b"tfhd" && traf_child.data.len() >= 8 {
                                // tfhd에서 track_ID 읽기 (4바이트 offset 후)
                                track_index = u32::from_be_bytes([
                                    traf_child.data[4],
                                    traf_child.data[5],
                                    traf_child.data[6],
                                    traf_child.data[7],
                                ]) as usize;
                                if track_index > 0 {
                                    track_index -= 1; // 1-based to 0-based
                                }
                                break;
                            }
                        }

                        // track_index에 맞게 배열 확장
                        while first_tfdts.len() <= track_index {
                            first_tfdts.push(None);
                        }

                        // tfdt 찾기
                        for traf_child in &traf_boxes {
                            if &traf_child.box_type == b"tfdt" && first_tfdts[track_index].is_none()
                            {
                                let tfdt_value = read_tfdt_value(&traf_child.data)?;
                                first_tfdts[track_index] = Some(tfdt_value);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(first_tfdts.into_iter().map(|v| v.unwrap_or(0)).collect())
}

/// Fragmented MP4의 총 duration 계산 (트랙별)
fn calculate_fragment_durations(
    boxes: &[BoxInfo],
    first_tfdt_offsets: &[u64],
) -> io::Result<Vec<u64>> {
    let mut max_decode_times: Vec<u64> = vec![0; first_tfdt_offsets.len()];
    let mut last_fragment_durations: Vec<u64> = vec![0; first_tfdt_offsets.len()];
    let mut prev_decode_times: Vec<Option<u64>> = vec![None; first_tfdt_offsets.len()];

    for box_info in boxes {
        if let BoxInfo::Small(mp4_box) = box_info {
            if &mp4_box.box_type == b"moof" {
                let moof_boxes = parse_container_box(&mp4_box.data)?;

                for moof_child in &moof_boxes {
                    if &moof_child.box_type == b"traf" {
                        let traf_boxes = parse_container_box(&moof_child.data)?;

                        // track_id 찾기
                        let mut track_index = 0;
                        for traf_child in &traf_boxes {
                            if &traf_child.box_type == b"tfhd" && traf_child.data.len() >= 8 {
                                track_index = u32::from_be_bytes([
                                    traf_child.data[4],
                                    traf_child.data[5],
                                    traf_child.data[6],
                                    traf_child.data[7],
                                ]) as usize;
                                if track_index > 0 {
                                    track_index -= 1;
                                }
                                break;
                            }
                        }

                        if track_index >= max_decode_times.len() {
                            continue;
                        }

                        // tfdt 읽기
                        let mut current_decode_time = 0u64;
                        for traf_child in &traf_boxes {
                            if &traf_child.box_type == b"tfdt" {
                                let raw_tfdt = read_tfdt_value(&traf_child.data)?;
                                current_decode_time = raw_tfdt;
                                // offset 적용
                                if track_index < first_tfdt_offsets.len() {
                                    current_decode_time = current_decode_time
                                        .saturating_sub(first_tfdt_offsets[track_index]);
                                }
                                break;
                            }
                        }

                        // trun에서 fragment의 duration 계산
                        let mut fragment_duration = 0u64;
                        for traf_child in &traf_boxes {
                            if &traf_child.box_type == b"trun" {
                                if let Ok(dur) = calculate_trun_duration(&traf_child.data) {
                                    fragment_duration = dur;
                                }
                                break;
                            }
                        }

                        // fragment duration을 이전 tfdt 차이로 추정 (trun에 없는 경우)
                        if fragment_duration == 0 {
                            if let Some(prev_time) = prev_decode_times[track_index] {
                                fragment_duration = current_decode_time.saturating_sub(prev_time);
                            }
                        }

                        // 이 fragment의 duration 기록 (다음 계산에 사용)
                        if fragment_duration > 0 {
                            last_fragment_durations[track_index] = fragment_duration;
                        }

                        // 현재 decode time 기록 (마지막 fragment의 시작 시간)
                        max_decode_times[track_index] = current_decode_time;
                        prev_decode_times[track_index] = Some(current_decode_time);
                    }
                }
            }
        }
    }

    // 마지막 fragment의 시작 시간 + 마지막 fragment의 duration
    for i in 0..max_decode_times.len() {
        if last_fragment_durations[i] > 0 {
            max_decode_times[i] += last_fragment_durations[i];
        }
    }

    Ok(max_decode_times)
}

/// trun 박스에서 fragment duration 계산
fn calculate_trun_duration(data: &[u8]) -> io::Result<u64> {
    if data.len() < 12 {
        return Ok(0);
    }

    let (_version, flags) = read_full_box_header(data)?;

    // flags 파싱
    let data_offset_present = (flags & 0x000001) != 0;
    let first_sample_flags_present = (flags & 0x000004) != 0;
    let sample_duration_present = (flags & 0x000100) != 0;
    let sample_size_present = (flags & 0x000200) != 0;
    let sample_flags_present = (flags & 0x000400) != 0;
    let sample_composition_time_present = (flags & 0x000800) != 0;

    let mut offset = 4; // version/flags 이후

    // sample_count 읽기
    if offset + 4 > data.len() {
        return Ok(0);
    }
    let sample_count = u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    // data_offset
    if data_offset_present {
        offset += 4;
    }

    // first_sample_flags
    if first_sample_flags_present {
        offset += 4;
    }

    // sample_duration_present인 경우 모든 샘플의 duration 합산
    if sample_duration_present {
        let mut total_duration = 0u64;

        for _ in 0..sample_count {
            if offset + 4 > data.len() {
                break;
            }
            let duration = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            total_duration += duration as u64;
            offset += 4;

            if sample_size_present {
                offset += 4;
            }
            if sample_flags_present {
                offset += 4;
            }
            if sample_composition_time_present {
                offset += 4;
            }
        }

        return Ok(total_duration);
    }

    Ok(0)
}

/// trun 박스에서 sample count 읽기
fn get_trun_sample_count(data: &[u8]) -> io::Result<u32> {
    if data.len() < 12 {
        return Ok(0);
    }

    // sample_count는 version/flags 다음 (offset 4-7)
    let sample_count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    Ok(sample_count)
}

/// tfdt 값 읽기
fn read_tfdt_value(data: &[u8]) -> io::Result<u64> {
    if data.len() < 8 {
        return Ok(0);
    }

    let (version, _) = read_full_box_header(data)?;

    if version == 1 && data.len() >= 12 {
        // 64비트
        Ok(u64::from_be_bytes([
            data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
        ]))
    } else if data.len() >= 8 {
        // 32비트
        Ok(u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as u64)
    } else {
        Ok(0)
    }
}

/// moof 박스 타임스탬프를 상대적으로 조정 (첫 번째 값 기준)
fn reset_moof_timestamps_relative(
    moof_data: &[u8],
    first_tfdt_offsets: &[u64],
) -> io::Result<Vec<u8>> {
    let moof_boxes = parse_container_box(moof_data)?;
    let mut output = Vec::new();

    for mp4_box in &moof_boxes {
        match &mp4_box.box_type {
            b"traf" => {
                let new_traf = reset_traf_timestamps_relative(&mp4_box.data, first_tfdt_offsets)?;
                write_box(&mut output, b"traf", &new_traf);
            }
            _ => {
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// traf 박스 타임스탬프를 상대적으로 조정
fn reset_traf_timestamps_relative(
    traf_data: &[u8],
    first_tfdt_offsets: &[u64],
) -> io::Result<Vec<u8>> {
    let traf_boxes = parse_container_box(traf_data)?;
    let mut output = Vec::new();

    // track_id 찾기
    let mut track_index = 0;
    for mp4_box in &traf_boxes {
        if &mp4_box.box_type == b"tfhd" && mp4_box.data.len() >= 8 {
            track_index = u32::from_be_bytes([
                mp4_box.data[4],
                mp4_box.data[5],
                mp4_box.data[6],
                mp4_box.data[7],
            ]) as usize;
            if track_index > 0 {
                track_index -= 1;
            }
            break;
        }
    }

    for mp4_box in &traf_boxes {
        match &mp4_box.box_type {
            b"tfdt" => {
                let offset = first_tfdt_offsets.get(track_index).copied().unwrap_or(0);
                let new_tfdt = reset_tfdt_relative(&mp4_box.data, offset)?;
                write_box(&mut output, b"tfdt", &new_tfdt);
            }
            _ => {
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// tfdt 박스 타임스탬프를 상대적으로 조정
fn reset_tfdt_relative(data: &[u8], offset: u64) -> io::Result<Vec<u8>> {
    let mut output = data.to_vec();
    let (version, _) = read_full_box_header(data)?;

    // 현재 값 읽기
    let current_value = read_tfdt_value(data)?;

    // offset을 빼서 상대적으로 만들기
    let new_value = current_value.saturating_sub(offset);

    if version == 1 && output.len() >= 12 {
        // version 1: baseMediaDecodeTime은 64비트
        let bytes = new_value.to_be_bytes();
        output[4..12].copy_from_slice(&bytes);
    } else if output.len() >= 8 {
        // version 0: baseMediaDecodeTime은 32비트
        let bytes = (new_value as u32).to_be_bytes();
        output[4..8].copy_from_slice(&bytes);
    }

    Ok(output)
}

/// traf 박스 타임스탬프 리셋 (구버전 - 호환성용)
fn reset_traf_timestamps(traf_data: &[u8]) -> io::Result<Vec<u8>> {
    let traf_boxes = parse_container_box(traf_data)?;
    let mut output = Vec::new();

    for mp4_box in &traf_boxes {
        match &mp4_box.box_type {
            b"tfdt" => {
                // tfdt (Track Fragment Decode Time) 리셋
                let new_tfdt = reset_tfdt(&mp4_box.data)?;
                write_box(&mut output, b"tfdt", &new_tfdt);
            }
            _ => {
                write_box(&mut output, &mp4_box.box_type, &mp4_box.data);
            }
        }
    }

    Ok(output)
}

/// tfdt 박스 타임스탬프 리셋
fn reset_tfdt(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut output = data.to_vec();
    let (version, _) = read_full_box_header(data)?;

    if version == 1 {
        // version 1: baseMediaDecodeTime은 64비트
        output[4..12].fill(0);
    } else {
        // version 0: baseMediaDecodeTime은 32비트
        output[4..8].fill(0);
    }

    Ok(output)
}

/// Fragmented MP4를 일반 MP4로 변환 (defragment)
pub fn defragment_mp4(data: &[u8]) -> io::Result<Vec<u8>> {
    let mp4 = parse_mp4(data)?;

    // Fragmented MP4인지 확인
    let has_fragments = mp4
        .all_boxes_in_order
        .iter()
        .any(|b| matches!(b, BoxInfo::Small(box_info) if &box_info.box_type == b"moof"));

    if !has_fragments {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "Not a fragmented MP4",
        ));
    }

    // moov에서 track 정보 추출
    let moov = mp4
        .moov
        .as_ref()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "moov box not found"))?;

    let mut track_fragments = extract_track_info_from_moov(&moov.data)?;

    // 모든 fragment에서 sample 데이터 수집
    collect_fragment_data(data, &mp4.all_boxes_in_order, &mut track_fragments)?;

    // 일반 MP4 생성
    build_regular_mp4(data, &mp4, track_fragments)
}

/// moov에서 track 정보 추출
fn extract_track_info_from_moov(moov_data: &[u8]) -> io::Result<Vec<TrackFragments>> {
    let moov_boxes = parse_container_box(moov_data)?;
    let mut tracks = Vec::new();

    for moov_box in &moov_boxes {
        if &moov_box.box_type == b"trak" {
            let trak_boxes = parse_container_box(&moov_box.data)?;

            // tkhd에서 track_id 추출
            let mut track_id = 0u32;
            for trak_box in &trak_boxes {
                if &trak_box.box_type == b"tkhd" && trak_box.data.len() >= 12 {
                    let version = trak_box.data[0];
                    let offset = if version == 1 { 20 } else { 12 };
                    if trak_box.data.len() >= offset + 4 {
                        track_id = u32::from_be_bytes([
                            trak_box.data[offset],
                            trak_box.data[offset + 1],
                            trak_box.data[offset + 2],
                            trak_box.data[offset + 3],
                        ]);
                    }
                    break;
                }
            }

            // mdia -> mdhd에서 timescale 추출
            let mut timescale = 1000u32;
            let mut codec_info = Vec::new();

            for trak_box in &trak_boxes {
                if &trak_box.box_type == b"mdia" {
                    let mdia_boxes = parse_container_box(&trak_box.data)?;

                    for mdia_box in &mdia_boxes {
                        if &mdia_box.box_type == b"mdhd" && mdia_box.data.len() >= 20 {
                            let version = mdia_box.data[0];
                            let offset = if version == 1 { 20 } else { 12 };
                            if mdia_box.data.len() >= offset + 4 {
                                timescale = u32::from_be_bytes([
                                    mdia_box.data[offset],
                                    mdia_box.data[offset + 1],
                                    mdia_box.data[offset + 2],
                                    mdia_box.data[offset + 3],
                                ]);
                            }
                        } else if &mdia_box.box_type == b"minf" {
                            // minf -> stbl -> stsd에서 codec 정보 추출
                            let minf_boxes = parse_container_box(&mdia_box.data)?;
                            for minf_child in &minf_boxes {
                                if &minf_child.box_type == b"stbl" {
                                    let stbl_boxes = parse_container_box(&minf_child.data)?;
                                    for stbl_child in &stbl_boxes {
                                        if &stbl_child.box_type == b"stsd" {
                                            codec_info = stbl_child.data.clone();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            tracks.push(TrackFragments {
                track_id,
                samples: Vec::new(),
                mdat_data: Vec::new(),
                timescale,
                codec_info,
            });
        }
    }

    Ok(tracks)
}

/// Fragment에서 sample 데이터 수집
fn collect_fragment_data(
    file_data: &[u8],
    boxes: &[BoxInfo],
    tracks: &mut [TrackFragments],
) -> io::Result<()> {
    let mut current_moof: Option<(&Mp4Box, usize)> = None; // (moof box, file offset)
    let mut current_mdat: Option<&[u8]> = None;

    // 파일 offset 계산
    let mut file_offset = 0usize;
    let mut box_list = Vec::new();

    for box_info in boxes {
        let size = match box_info {
            BoxInfo::Small(b) => b.data.len() + 8,
            BoxInfo::Mdat(m) => m.data_size + 8,
        };

        box_list.push((file_offset, box_info));
        file_offset += size;
    }

    // moof와 mdat 처리
    for (offset, box_info) in &box_list {
        match box_info {
            BoxInfo::Small(mp4_box) if &mp4_box.box_type == b"moof" => {
                current_moof = Some((mp4_box, *offset));
            }
            BoxInfo::Mdat(mdat) => {
                current_mdat =
                    Some(&file_data[mdat.data_offset..mdat.data_offset + mdat.data_size]);

                // moof와 mdat 쌍 처리
                if let Some((moof_box, moof_offset)) = current_moof {
                    let moof_size = moof_box.data.len() + 8;
                    let mdat_start_in_file = moof_offset + moof_size + 8; // moof + mdat header

                    let moof_boxes = parse_container_box(&moof_box.data)?;

                    for moof_child in &moof_boxes {
                        if &moof_child.box_type == b"traf" {
                            process_traf_with_offset(
                                &moof_child.data,
                                tracks,
                                &current_mdat,
                                moof_offset,
                                mdat_start_in_file,
                            )?;
                        }
                    }

                    current_moof = None;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

/// traf 박스 처리 (data_offset 기반)
fn process_traf_with_offset(
    traf_data: &[u8],
    tracks: &mut [TrackFragments],
    current_mdat: &Option<&[u8]>,
    moof_offset: usize,
    mdat_start_in_file: usize,
) -> io::Result<()> {
    let traf_boxes = parse_container_box(traf_data)?;

    // tfhd에서 track_id와 기본값 추출
    let mut track_id = 0u32;
    let mut default_duration = 0u32;
    let mut default_size = 0u32;
    let mut default_flags = 0u32;

    for traf_box in &traf_boxes {
        if &traf_box.box_type == b"tfhd" && traf_box.data.len() >= 8 {
            let flags =
                u32::from_be_bytes([0, traf_box.data[1], traf_box.data[2], traf_box.data[3]]);

            track_id = u32::from_be_bytes([
                traf_box.data[4],
                traf_box.data[5],
                traf_box.data[6],
                traf_box.data[7],
            ]);

            let mut pos = 8;

            // base_data_offset (0x000001)
            if flags & 0x000001 != 0 {
                pos += 8;
            }

            // sample_description_index (0x000002)
            if flags & 0x000002 != 0 {
                pos += 4;
            }

            // default_sample_duration (0x000008)
            if flags & 0x000008 != 0 && traf_box.data.len() >= pos + 4 {
                default_duration = u32::from_be_bytes([
                    traf_box.data[pos],
                    traf_box.data[pos + 1],
                    traf_box.data[pos + 2],
                    traf_box.data[pos + 3],
                ]);
                pos += 4;
            }

            // default_sample_size (0x000010)
            if flags & 0x000010 != 0 && traf_box.data.len() >= pos + 4 {
                default_size = u32::from_be_bytes([
                    traf_box.data[pos],
                    traf_box.data[pos + 1],
                    traf_box.data[pos + 2],
                    traf_box.data[pos + 3],
                ]);
                pos += 4;
            }

            // default_sample_flags (0x000020)
            if flags & 0x000020 != 0 && traf_box.data.len() >= pos + 4 {
                default_flags = u32::from_be_bytes([
                    traf_box.data[pos],
                    traf_box.data[pos + 1],
                    traf_box.data[pos + 2],
                    traf_box.data[pos + 3],
                ]);
            }
        }
    }

    // trun에서 sample 정보 추출하고 mdat에서 데이터 읽기
    for traf_box in &traf_boxes {
        if &traf_box.box_type == b"trun" {
            let (samples, data_offset_opt) = parse_trun_samples(
                &traf_box.data,
                default_duration,
                default_size,
                default_flags,
            )?;

            // track 찾기
            if let Some(track) = tracks.iter_mut().find(|t| t.track_id == track_id) {
                // mdat 데이터 복사 - data_offset 사용
                if let Some(mdat_data) = current_mdat {
                    if let Some(data_offset) = data_offset_opt {
                        // data_offset은 moof 시작에서의 상대 offset
                        // 실제 mdat 내 offset = (moof_offset + data_offset) - mdat_start_in_file
                        let absolute_offset = (moof_offset as i32 + data_offset) as usize;
                        let mdat_offset = absolute_offset.saturating_sub(mdat_start_in_file);

                        let mut offset = mdat_offset;

                        // 모든 sample 데이터를 순서대로 복사
                        for sample in &samples {
                            let size = sample.size as usize;
                            if offset + size <= mdat_data.len() {
                                track
                                    .mdat_data
                                    .extend_from_slice(&mdat_data[offset..offset + size]);
                                offset += size;
                            }
                        }
                    }
                }

                track.samples.extend(samples);
            }
        }
    }

    Ok(())
}

/// trun에서 sample 정보 파싱 (data_offset 포함)
fn parse_trun_samples(
    trun_data: &[u8],
    default_duration: u32,
    default_size: u32,
    default_flags: u32,
) -> io::Result<(Vec<FragmentSampleInfo>, Option<i32>)> {
    if trun_data.len() < 8 {
        return Ok((Vec::new(), None));
    }

    let flags = u32::from_be_bytes([0, trun_data[1], trun_data[2], trun_data[3]]);
    let sample_count =
        u32::from_be_bytes([trun_data[4], trun_data[5], trun_data[6], trun_data[7]]) as usize;

    let mut pos = 8;
    let mut data_offset = None;

    // data_offset (0x000001)
    if flags & 0x000001 != 0 && trun_data.len() >= pos + 4 {
        data_offset = Some(i32::from_be_bytes([
            trun_data[pos],
            trun_data[pos + 1],
            trun_data[pos + 2],
            trun_data[pos + 3],
        ]));
        pos += 4;
    }

    // first_sample_flags (0x000004)
    if flags & 0x000004 != 0 {
        pos += 4;
    }

    let has_duration = flags & 0x000100 != 0;
    let has_size = flags & 0x000200 != 0;
    let has_flags = flags & 0x000400 != 0;
    let has_composition = flags & 0x000800 != 0;

    let mut samples = Vec::with_capacity(sample_count);

    for _ in 0..sample_count {
        let mut duration = default_duration;
        let mut size = default_size;
        let mut sample_flags = default_flags;
        let mut composition = 0i32;

        if has_duration && trun_data.len() >= pos + 4 {
            duration = u32::from_be_bytes([
                trun_data[pos],
                trun_data[pos + 1],
                trun_data[pos + 2],
                trun_data[pos + 3],
            ]);
            pos += 4;
        }

        if has_size && trun_data.len() >= pos + 4 {
            size = u32::from_be_bytes([
                trun_data[pos],
                trun_data[pos + 1],
                trun_data[pos + 2],
                trun_data[pos + 3],
            ]);
            pos += 4;
        }

        if has_flags && trun_data.len() >= pos + 4 {
            sample_flags = u32::from_be_bytes([
                trun_data[pos],
                trun_data[pos + 1],
                trun_data[pos + 2],
                trun_data[pos + 3],
            ]);
            pos += 4;
        }

        if has_composition && trun_data.len() >= pos + 4 {
            composition = i32::from_be_bytes([
                trun_data[pos],
                trun_data[pos + 1],
                trun_data[pos + 2],
                trun_data[pos + 3],
            ]);
            pos += 4;
        }

        samples.push(FragmentSampleInfo {
            duration,
            size,
            flags: sample_flags,
            composition_time_offset: composition,
        });
    }

    Ok((samples, data_offset))
}

/// 일반 MP4 빌드
fn build_regular_mp4(
    _original_data: &[u8],
    mp4: &Mp4File,
    track_fragments: Vec<TrackFragments>,
) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // ftyp 박스 생성 (QuickTime 호환성을 위해 isom 기반으로)
    let ftyp_data = {
        let mut data = Vec::new();
        data.extend_from_slice(b"isom"); // major_brand
        data.extend_from_slice(&[0, 0, 0, 2]); // minor_version
                                               // compatible_brands: isom, iso2, avc1, mp41
        data.extend_from_slice(b"isom");
        data.extend_from_slice(b"iso2");
        data.extend_from_slice(b"avc1");
        data.extend_from_slice(b"mp41");
        data
    };
    write_box(&mut output, b"ftyp", &ftyp_data);

    // moov의 크기를 먼저 계산해야 mdat offset을 알 수 있음
    // 임시로 moov를 생성해서 크기 계산
    let temp_moov = build_regular_moov(
        &mp4.moov.as_ref().unwrap().data,
        &track_fragments,
        &Vec::new(),
    )?;
    let moov_size = temp_moov.len() + 8; // box header 포함

    // 각 트랙의 mdat 시작 offset 계산
    let ftyp_size = 32; // 고정된 ftyp 크기 (isom + 4 brands)

    let mut track_mdat_offsets = Vec::new();
    let mut current_offset = ftyp_size + moov_size + 8; // ftyp + moov + mdat header

    for track in &track_fragments {
        track_mdat_offsets.push(current_offset);
        current_offset += track.mdat_data.len();
    }

    // 실제 moov 생성 (offset 정보 포함)
    let moov_data = build_regular_moov(
        &mp4.moov.as_ref().unwrap().data,
        &track_fragments,
        &track_mdat_offsets,
    )?;
    write_box(&mut output, b"moov", &moov_data);

    // 단일 mdat에 모든 트랙 데이터 저장 (트랙 순서대로)
    let mut mdat_data = Vec::new();
    for track in &track_fragments {
        mdat_data.extend_from_slice(&track.mdat_data);
    }
    write_box(&mut output, b"mdat", &mdat_data);

    Ok(output)
}

/// 일반 MP4용 moov 박스 생성
fn build_regular_moov(
    original_moov_data: &[u8],
    track_fragments: &[TrackFragments],
    track_mdat_offsets: &[usize],
) -> io::Result<Vec<u8>> {
    let moov_boxes = parse_container_box(original_moov_data)?;
    let mut output = Vec::new();

    // mvhd에서 movie timescale 추출
    let movie_timescale = if let Some(mvhd_box) = moov_boxes.iter().find(|b| &b.box_type == b"mvhd")
    {
        if mvhd_box.data.len() >= 16 {
            let version = mvhd_box.data[0];
            if version == 1 && mvhd_box.data.len() >= 24 {
                u32::from_be_bytes([
                    mvhd_box.data[20],
                    mvhd_box.data[21],
                    mvhd_box.data[22],
                    mvhd_box.data[23],
                ])
            } else if mvhd_box.data.len() >= 16 {
                u32::from_be_bytes([
                    mvhd_box.data[12],
                    mvhd_box.data[13],
                    mvhd_box.data[14],
                    mvhd_box.data[15],
                ])
            } else {
                1000 // fallback
            }
        } else {
            1000 // fallback
        }
    } else {
        1000 // fallback
    };

    for moov_box in &moov_boxes {
        match &moov_box.box_type {
            b"mvhd" => {
                // mvhd duration: track duration을 movie timescale로 변환
                // 가장 긴 트랙의 duration을 사용 (일반적으로 비디오 트랙)
                let max_duration_in_movie_ts = track_fragments
                    .iter()
                    .map(|t| {
                        let track_duration: u64 = t.samples.iter().map(|s| s.duration as u64).sum();
                        // track timescale -> movie timescale로 변환
                        if t.timescale != 0 && t.timescale != movie_timescale {
                            (track_duration * movie_timescale as u64) / t.timescale as u64
                        } else {
                            track_duration
                        }
                    })
                    .max()
                    .unwrap_or(0);

                let new_mvhd = update_mvhd_duration(&moov_box.data, max_duration_in_movie_ts)?;
                write_box(&mut output, b"mvhd", &new_mvhd);
            }
            b"trak" => {
                // 원본 trak에서 track_id 추출
                let trak_boxes = parse_container_box(&moov_box.data)?;
                let mut trak_track_id = 0u32;

                for trak_box in &trak_boxes {
                    if &trak_box.box_type == b"tkhd" && trak_box.data.len() >= 12 {
                        let version = trak_box.data[0];
                        let offset = if version == 1 { 20 } else { 12 };
                        if trak_box.data.len() >= offset + 4 {
                            trak_track_id = u32::from_be_bytes([
                                trak_box.data[offset],
                                trak_box.data[offset + 1],
                                trak_box.data[offset + 2],
                                trak_box.data[offset + 3],
                            ]);
                        }
                        break;
                    }
                }

                // track_id로 해당하는 fragment 찾기
                if let Some(track_frag_index) = track_fragments
                    .iter()
                    .position(|t| t.track_id == trak_track_id)
                {
                    let mdat_offset = if track_frag_index < track_mdat_offsets.len() {
                        track_mdat_offsets[track_frag_index]
                    } else {
                        8000 // fallback
                    };

                    let new_trak = build_regular_trak(
                        &moov_box.data,
                        &track_fragments[track_frag_index],
                        mdat_offset,
                    )?;
                    write_box(&mut output, b"trak", &new_trak);
                } else {
                    // fragment가 없으면 원본 그대로
                    write_box(&mut output, b"trak", &moov_box.data);
                }
            }
            _ => {
                // 다른 박스는 그대로
                write_box(&mut output, &moov_box.box_type, &moov_box.data);
            }
        }
    }

    Ok(output)
}

/// mvhd duration 업데이트 (version 0으로 다운그레이드)
fn update_mvhd_duration(mvhd_data: &[u8], duration: u64) -> io::Result<Vec<u8>> {
    let old_version = mvhd_data[0];

    if old_version == 1 && mvhd_data.len() >= 32 {
        // version 1 -> version 0으로 다운그레이드
        let mut output = vec![0u8; 108]; // mvhd version 0의 표준 크기

        // version = 0, flags = 0
        output[0..4].copy_from_slice(&[0, 0, 0, 0]);

        // creation_time = 0 (4바이트)
        output[4..8].copy_from_slice(&[0, 0, 0, 0]);

        // modification_time = 0 (4바이트)
        output[8..12].copy_from_slice(&[0, 0, 0, 0]);

        // timescale (4바이트) - 원본에서 복사
        output[12..16].copy_from_slice(&mvhd_data[20..24]);

        // duration (4바이트) - 새 값 설정
        let duration_32 = if duration > u32::MAX as u64 {
            u32::MAX
        } else {
            duration as u32
        };
        output[16..20].copy_from_slice(&duration_32.to_be_bytes());

        // rate (4바이트) - 0x00010000 (1.0)
        output[20..24].copy_from_slice(&[0x00, 0x01, 0x00, 0x00]);

        // volume (2바이트) - 0x0100 (1.0)
        output[24..26].copy_from_slice(&[0x01, 0x00]);

        // reserved (10바이트)
        output[26..36].fill(0);

        // matrix (36바이트) - 단위 행렬
        output[36..40].copy_from_slice(&[0x00, 0x01, 0x00, 0x00]);
        output[40..44].fill(0);
        output[44..48].fill(0);
        output[48..52].fill(0);
        output[52..56].copy_from_slice(&[0x00, 0x01, 0x00, 0x00]);
        output[56..60].fill(0);
        output[60..64].fill(0);
        output[64..68].fill(0);
        output[68..72].copy_from_slice(&[0x40, 0x00, 0x00, 0x00]);

        // pre_defined (24바이트)
        output[72..96].fill(0);

        // next_track_ID - 원본 또는 3
        if mvhd_data.len() >= 112 {
            output[96..100].copy_from_slice(&mvhd_data[108..112]);
        } else {
            output[96..100].copy_from_slice(&[0, 0, 0, 3]);
        }

        // 나머지는 0
        output[100..108].fill(0);

        return Ok(output);
    }

    // 이미 version 0이면 creation_time, modification_time, duration만 업데이트
    let mut output = mvhd_data.to_vec();

    if output.len() >= 20 {
        // creation_time = 0
        output[4..8].fill(0);
        // modification_time = 0
        output[8..12].fill(0);
        // duration 설정
        let bytes = (duration as u32).to_be_bytes();
        output[16..20].copy_from_slice(&bytes);
    }

    Ok(output)
}

/// 일반 MP4용 trak 박스 생성
fn build_regular_trak(
    original_trak_data: &[u8],
    track_fragment: &TrackFragments,
    mdat_offset: usize,
) -> io::Result<Vec<u8>> {
    let trak_boxes = parse_container_box(original_trak_data)?;
    let mut output = Vec::new();

    // track의 총 duration 계산
    let total_duration: u64 = track_fragment
        .samples
        .iter()
        .map(|s| s.duration as u64)
        .sum();

    for trak_box in &trak_boxes {
        match &trak_box.box_type {
            b"tkhd" => {
                // tkhd duration 업데이트
                let new_tkhd = update_tkhd_duration(&trak_box.data, total_duration)?;
                write_box(&mut output, b"tkhd", &new_tkhd);
            }
            b"mdia" => {
                // mdia 재구성
                let new_mdia = build_regular_mdia(&trak_box.data, track_fragment, mdat_offset)?;
                write_box(&mut output, b"mdia", &new_mdia);
            }
            b"edts" => {
                // edts는 유지 (QuickTime Player 호환성)
                write_box(&mut output, &trak_box.box_type, &trak_box.data);
            }
            _ => {
                write_box(&mut output, &trak_box.box_type, &trak_box.data);
            }
        }
    }

    Ok(output)
}

/// tkhd duration 업데이트 (version 0으로 다운그레이드)
fn update_tkhd_duration(tkhd_data: &[u8], duration: u64) -> io::Result<Vec<u8>> {
    let old_version = tkhd_data[0];

    if old_version == 1 && tkhd_data.len() >= 36 {
        // version 1 -> version 0으로 다운그레이드
        let mut output = vec![0u8; 92]; // tkhd version 0의 표준 크기

        // version = 0, flags (원본 flags 복사)
        output[0] = 0;
        output[1..4].copy_from_slice(&tkhd_data[1..4]);

        // creation_time = 0 (4바이트)
        output[4..8].fill(0);

        // modification_time = 0 (4바이트)
        output[8..12].fill(0);

        // track_id (4바이트) - 원본에서 복사
        output[12..16].copy_from_slice(&tkhd_data[20..24]);

        // reserved (4바이트)
        output[16..20].fill(0);

        // duration (4바이트) - 새 값 설정
        let duration_32 = if duration > u32::MAX as u64 {
            u32::MAX
        } else {
            duration as u32
        };
        output[20..24].copy_from_slice(&duration_32.to_be_bytes());

        // 나머지는 원본에서 복사 (layer, alternate_group, volume, reserved, matrix, width, height)
        if tkhd_data.len() >= 104 {
            // version 1의 36바이트 이후 데이터를 version 0의 24바이트 이후로 복사
            output[24..92].copy_from_slice(&tkhd_data[36..104]);
        }

        return Ok(output);
    }

    // 이미 version 0이면 creation_time, modification_time, duration만 업데이트
    let mut output = tkhd_data.to_vec();

    if output.len() >= 24 {
        // creation_time = 0
        output[4..8].fill(0);
        // modification_time = 0
        output[8..12].fill(0);
        // duration 설정
        let bytes = (duration as u32).to_be_bytes();
        output[20..24].copy_from_slice(&bytes);
    }

    Ok(output)
}

/// 일반 MP4용 mdia 박스 생성/// 일반 MP4용 mdia 박스 생성
fn build_regular_mdia(
    original_mdia_data: &[u8],
    track_fragment: &TrackFragments,
    mdat_offset: usize,
) -> io::Result<Vec<u8>> {
    let mdia_boxes = parse_container_box(original_mdia_data)?;
    let mut output = Vec::new();

    let total_duration: u64 = track_fragment
        .samples
        .iter()
        .map(|s| s.duration as u64)
        .sum();

    for mdia_box in &mdia_boxes {
        match &mdia_box.box_type {
            b"mdhd" => {
                // mdhd duration 업데이트
                let new_mdhd = update_mdhd_duration(&mdia_box.data, total_duration)?;
                write_box(&mut output, b"mdhd", &new_mdhd);
            }
            b"minf" => {
                // minf 재구성 (stbl 업데이트)
                let new_minf = build_regular_minf(&mdia_box.data, track_fragment, mdat_offset)?;
                write_box(&mut output, b"minf", &new_minf);
            }
            _ => {
                write_box(&mut output, &mdia_box.box_type, &mdia_box.data);
            }
        }
    }

    Ok(output)
}

/// mdhd duration 업데이트 (version 0으로 다운그레이드)
fn update_mdhd_duration(mdhd_data: &[u8], duration: u64) -> io::Result<Vec<u8>> {
    let old_version = mdhd_data[0];

    if old_version == 1 && mdhd_data.len() >= 32 {
        // version 1 -> version 0으로 다운그레이드
        let mut output = vec![0u8; 32]; // mdhd version 0의 표준 크기

        // version = 0, flags (원본 flags 복사)
        output[0] = 0;
        output[1..4].copy_from_slice(&mdhd_data[1..4]);

        // creation_time = 0 (4바이트)
        output[4..8].fill(0);

        // modification_time = 0 (4바이트)
        output[8..12].fill(0);

        // timescale (4바이트) - 원본에서 복사
        output[12..16].copy_from_slice(&mdhd_data[20..24]);

        // duration (4바이트) - 새 값 설정
        let duration_32 = if duration > u32::MAX as u64 {
            u32::MAX
        } else {
            duration as u32
        };
        output[16..20].copy_from_slice(&duration_32.to_be_bytes());

        // language + pre_defined (4바이트) - 원본에서 복사
        if mdhd_data.len() >= 36 {
            output[20..24].copy_from_slice(&mdhd_data[32..36]);
        }

        // 나머지는 원본에서 복사
        if mdhd_data.len() >= 44 {
            output[24..32].copy_from_slice(&mdhd_data[36..44]);
        }

        return Ok(output);
    }

    // 이미 version 0이면 creation_time, modification_time, duration만 업데이트
    let mut output = mdhd_data.to_vec();

    if output.len() >= 20 {
        // creation_time = 0
        output[4..8].fill(0);
        // modification_time = 0
        output[8..12].fill(0);
        // duration 설정
        let bytes = (duration as u32).to_be_bytes();
        output[16..20].copy_from_slice(&bytes);
    }

    Ok(output)
}

/// 일반 MP4용 minf 박스 생성
fn build_regular_minf(
    original_minf_data: &[u8],
    track_fragment: &TrackFragments,
    mdat_offset: usize,
) -> io::Result<Vec<u8>> {
    let minf_boxes = parse_container_box(original_minf_data)?;
    let mut output = Vec::new();

    for minf_box in &minf_boxes {
        match &minf_box.box_type {
            b"stbl" => {
                // stbl 재구성 (sample tables 생성)
                let new_stbl = build_sample_tables(track_fragment, mdat_offset)?;
                write_box(&mut output, b"stbl", &new_stbl);
            }
            _ => {
                write_box(&mut output, &minf_box.box_type, &minf_box.data);
            }
        }
    }

    Ok(output)
}

/// Sample Tables 생성 (stts, stsz, stsc, stco, ctts)
fn build_sample_tables(track_fragment: &TrackFragments, mdat_offset: usize) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // stsd (Sample Description) - 원본 codec 정보 사용
    if !track_fragment.codec_info.is_empty() {
        write_box(&mut output, b"stsd", &track_fragment.codec_info);
    }

    // stts (Time To Sample)
    let stts = build_stts(&track_fragment.samples)?;
    write_box(&mut output, b"stts", &stts);

    // stsc (Sample To Chunk) - 각 sample을 별도의 chunk로
    let stsc = build_stsc_multi_chunk(track_fragment.samples.len())?;
    write_box(&mut output, b"stsc", &stsc);

    // stsz (Sample Size)
    let stsz = build_stsz(&track_fragment.samples)?;
    write_box(&mut output, b"stsz", &stsz);

    // stco (Chunk Offset) - 각 sample의 offset 계산
    let stco = build_stco_multi_chunk(&track_fragment.samples, mdat_offset)?;
    write_box(&mut output, b"stco", &stco);

    // ctts (Composition Time To Sample) - composition offset이 있으면
    if track_fragment
        .samples
        .iter()
        .any(|s| s.composition_time_offset != 0)
    {
        let ctts = build_ctts(&track_fragment.samples)?;
        write_box(&mut output, b"ctts", &ctts);
    }

    // stss (Sync Sample) - keyframe 정보
    let stss = build_stss(&track_fragment.samples)?;
    if !stss.is_empty() {
        write_box(&mut output, b"stss", &stss);
    }

    Ok(output)
}

/// stts (Time To Sample) 생성
fn build_stts(samples: &[FragmentSampleInfo]) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // version (1) + flags (3)
    output.extend_from_slice(&[0, 0, 0, 0]);

    // entry_count를 위한 공간 (나중에 채움)
    let entry_count_pos = output.len();
    output.extend_from_slice(&[0, 0, 0, 0]);

    let mut entry_count = 0u32;
    let mut last_duration = 0u32;
    let mut sample_count = 0u32;

    for sample in samples {
        if sample.duration == last_duration && sample_count > 0 {
            sample_count += 1;
        } else {
            if sample_count > 0 {
                output.extend_from_slice(&sample_count.to_be_bytes());
                output.extend_from_slice(&last_duration.to_be_bytes());
                entry_count += 1;
            }
            last_duration = sample.duration;
            sample_count = 1;
        }
    }

    // 마지막 entry
    if sample_count > 0 {
        output.extend_from_slice(&sample_count.to_be_bytes());
        output.extend_from_slice(&last_duration.to_be_bytes());
        entry_count += 1;
    }

    // entry_count 채우기
    output[entry_count_pos..entry_count_pos + 4].copy_from_slice(&entry_count.to_be_bytes());

    Ok(output)
}

/// stsc (Sample To Chunk) 생성 - 각 sample을 별도의 chunk로
fn build_stsc_multi_chunk(_sample_count: usize) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // version (1) + flags (3)
    output.extend_from_slice(&[0, 0, 0, 0]);

    // entry_count = 1 (모든 chunk가 1개의 sample을 가짐)
    output.extend_from_slice(&[0, 0, 0, 1]);

    // first_chunk = 1
    output.extend_from_slice(&[0, 0, 0, 1]);

    // samples_per_chunk = 1
    output.extend_from_slice(&[0, 0, 0, 1]);

    // sample_description_index = 1
    output.extend_from_slice(&[0, 0, 0, 1]);

    Ok(output)
}

/// stsz (Sample Size) 생성
fn build_stsz(samples: &[FragmentSampleInfo]) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // version (1) + flags (3)
    output.extend_from_slice(&[0, 0, 0, 0]);

    // sample_size = 0 (가변 크기)
    output.extend_from_slice(&[0, 0, 0, 0]);

    // sample_count
    output.extend_from_slice(&(samples.len() as u32).to_be_bytes());

    // 각 sample의 크기
    for sample in samples {
        output.extend_from_slice(&sample.size.to_be_bytes());
    }

    Ok(output)
}

/// stco (Chunk Offset) 생성 - 각 sample마다 offset 계산
fn build_stco_multi_chunk(
    samples: &[FragmentSampleInfo],
    mdat_start_offset: usize,
) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // version (1) + flags (3)
    output.extend_from_slice(&[0, 0, 0, 0]);

    // entry_count = sample 개수 (각 sample이 하나의 chunk)
    output.extend_from_slice(&(samples.len() as u32).to_be_bytes());

    // 각 sample의 offset 계산
    let mut current_offset = mdat_start_offset;
    for sample in samples {
        output.extend_from_slice(&(current_offset as u32).to_be_bytes());
        current_offset += sample.size as usize;
    }

    Ok(output)
}

/// ctts (Composition Time To Sample) 생성
fn build_ctts(samples: &[FragmentSampleInfo]) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // version (1) + flags (3)
    output.extend_from_slice(&[0, 0, 0, 0]);

    // entry_count를 위한 공간
    let entry_count_pos = output.len();
    output.extend_from_slice(&[0, 0, 0, 0]);

    let mut entry_count = 0u32;
    let mut last_offset = 0i32;
    let mut sample_count = 0u32;

    for sample in samples {
        if sample.composition_time_offset == last_offset && sample_count > 0 {
            sample_count += 1;
        } else {
            if sample_count > 0 {
                output.extend_from_slice(&sample_count.to_be_bytes());
                output.extend_from_slice(&last_offset.to_be_bytes());
                entry_count += 1;
            }
            last_offset = sample.composition_time_offset;
            sample_count = 1;
        }
    }

    // 마지막 entry
    if sample_count > 0 {
        output.extend_from_slice(&sample_count.to_be_bytes());
        output.extend_from_slice(&last_offset.to_be_bytes());
        entry_count += 1;
    }

    // entry_count 채우기
    output[entry_count_pos..entry_count_pos + 4].copy_from_slice(&entry_count.to_be_bytes());

    Ok(output)
}

/// stss (Sync Sample) 생성 - keyframe만
fn build_stss(samples: &[FragmentSampleInfo]) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut keyframes = Vec::new();

    for (idx, sample) in samples.iter().enumerate() {
        // sample_depends_on == 2 means I-frame (sync sample)
        // flags의 bit 24-25가 sample_depends_on
        let depends_on = (sample.flags >> 24) & 0x3;
        if depends_on == 2 || depends_on == 0 {
            keyframes.push((idx + 1) as u32); // 1-based index
        }
    }

    if keyframes.is_empty() {
        return Ok(output);
    }

    // version (1) + flags (3)
    output.extend_from_slice(&[0, 0, 0, 0]);

    // entry_count
    output.extend_from_slice(&(keyframes.len() as u32).to_be_bytes());

    // sample_number (1-based)
    for keyframe in keyframes {
        output.extend_from_slice(&keyframe.to_be_bytes());
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mp4_reader() {
        let data = vec![0x00, 0x00, 0x00, 0x01, 0x12, 0x34, 0x56, 0x78];
        let mut reader = Mp4Reader::new(&data);

        assert_eq!(reader.read_u32().unwrap(), 1);
        assert_eq!(reader.read_u32().unwrap(), 0x12345678);
    }

    #[test]
    fn test_full_box_header() {
        let data = vec![0x01, 0x00, 0x00, 0x03]; // version=1, flags=3
        let (version, flags) = read_full_box_header(&data).unwrap();

        assert_eq!(version, 1);
        assert_eq!(flags, 3);
    }
}
