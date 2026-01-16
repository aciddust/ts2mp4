use crate::ts_parser::MediaData;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone)]
pub struct RgbFrame {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>, // RGB 데이터 (width * height * 3)
    pub pts: Option<u64>,
}

/// H.264 NAL unit에서 I-frame들을 추출
/// 주의: 실제 디코딩이 아닌 I-frame NAL units만 추출
pub fn extract_iframes(media_data: &MediaData) -> io::Result<Vec<Vec<u8>>> {
    if media_data.video_stream.is_empty() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "No video data found",
        ));
    }

    let mut iframes = Vec::new();
    let data = &media_data.video_stream;
    let mut offset = 0;

    while offset < data.len() {
        // Find NAL unit start code
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

        // NAL type 5 is IDR (I-frame)
        if nal_type == 5 {
            // Find the end of this frame
            let mut frame_end = nal_start + 1;

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

            // Build complete frame with SPS/PPS
            let mut frame_data = Vec::new();
            if let Some(ref sps) = media_data.sps {
                frame_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
                frame_data.extend_from_slice(sps);
            }
            if let Some(ref pps) = media_data.pps {
                frame_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
                frame_data.extend_from_slice(pps);
            }
            frame_data.extend_from_slice(&data[offset..frame_end]);

            iframes.push(frame_data);
            offset = frame_end;
        } else {
            offset += 1;
        }
    }

    Ok(iframes)
}

/// 간단한 더미 RGB 프레임 생성 (실제 디코딩 없이 테스트용)
/// 실제로는 외부 디코더(ffmpeg 등)가 필요합니다
pub fn create_placeholder_frames(
    media_data: &MediaData,
    count: usize,
) -> io::Result<Vec<RgbFrame>> {
    let mut frames = Vec::new();
    let width = media_data.width;
    let height = media_data.height;

    for i in 0..count {
        // 그라디언트 패턴 생성
        let mut data = Vec::with_capacity((width * height * 3) as usize);

        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 255.0) as u8;
                let g = ((y as f32 / height as f32) * 255.0) as u8;
                let b = ((i as f32 / count as f32) * 255.0) as u8;

                data.push(r);
                data.push(g);
                data.push(b);
            }
        }

        frames.push(RgbFrame {
            width,
            height,
            data,
            pts: None,
        });
    }

    Ok(frames)
}

/// H.264 NAL unit 데이터를 외부 디코더로 디코딩
/// (ffmpeg, openh264 등을 std::process::Command로 호출)
#[allow(dead_code)]
fn decode_h264_with_external_tool(
    _nal_data: &[u8],
    _width: u16,
    _height: u16,
) -> io::Result<RgbFrame> {
    // 예시: ffmpeg를 외부 프로세스로 호출하여 디코딩
    // ffmpeg -f h264 -i input.h264 -f rawvideo -pix_fmt rgb24 output.rgb

    // 실제 구현 시:
    // 1. 임시 파일에 NAL data 저장
    // 2. ffmpeg 실행
    // 3. RGB 데이터 읽기
    // 4. 임시 파일 삭제

    Err(io::Error::new(
        ErrorKind::Unsupported,
        "External decoder not implemented",
    ))
}
