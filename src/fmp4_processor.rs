use std::io::{self, ErrorKind};

/// Fragmented MP4 스트리밍 프로세서
/// 초기화 세그먼트(m4s)와 미디어 세그먼트(m4v)를 실시간으로 처리
#[derive(Debug)]
pub struct FragmentedMP4Processor {
    init_segment: Vec<u8>,
    base_decode_time: Option<u64>,
}

impl FragmentedMP4Processor {
    pub fn new() -> Self {
        Self {
            init_segment: Vec::new(),
            base_decode_time: None,
        }
    }

    /// 초기화 세그먼트 설정 (m4s 파일)
    pub fn set_init_segment(&mut self, data: &[u8]) -> io::Result<()> {
        self.init_segment = data.to_vec();
        Ok(())
    }

    /// 미디어 세그먼트 처리 (m4v 파일)
    /// 첫 세그먼트의 타임스탬프를 base로 설정하고, 이후 세그먼트들의 타임스탬프를 조정
    pub fn process_segment(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        let mut result = Vec::with_capacity(data.len());
        let mut offset = 0;

        while offset < data.len() {
            if offset + 8 > data.len() {
                break;
            }

            let (box_type, box_size) = self.parse_box_header(&data[offset..])?;

            if offset + box_size > data.len() {
                break;
            }

            let box_data = &data[offset..offset + box_size];

            match &box_type {
                b"moof" => {
                    // moof 박스 내 tfdt의 타임스탬프 조정
                    let adjusted_moof = self.adjust_moof_timestamps(box_data)?;
                    result.extend_from_slice(&adjusted_moof);
                }
                _ => {
                    // 다른 박스는 그대로 복사
                    result.extend_from_slice(box_data);
                }
            }

            offset += box_size;
        }

        Ok(result)
    }

    fn parse_box_header(&self, data: &[u8]) -> io::Result<([u8; 4], usize)> {
        if data.len() < 8 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Data too short for box header",
            ));
        }

        let size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let box_type = [data[4], data[5], data[6], data[7]];

        if size < 8 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid box size"));
        }

        Ok((box_type, size))
    }

    fn adjust_moof_timestamps(&mut self, moof_data: &[u8]) -> io::Result<Vec<u8>> {
        let mut result = Vec::with_capacity(moof_data.len());
        let mut offset = 8; // moof 헤더 건너뛰기

        // moof 헤더 복사
        result.extend_from_slice(&moof_data[0..8]);

        while offset < moof_data.len() {
            if offset + 8 > moof_data.len() {
                break;
            }

            let (box_type, box_size) = self.parse_box_header(&moof_data[offset..])?;

            if offset + box_size > moof_data.len() {
                break;
            }

            let box_data = &moof_data[offset..offset + box_size];

            match &box_type {
                b"traf" => {
                    // traf 박스 처리
                    let adjusted_traf = self.adjust_traf_timestamps(box_data)?;
                    result.extend_from_slice(&adjusted_traf);
                }
                _ => {
                    result.extend_from_slice(box_data);
                }
            }

            offset += box_size;
        }

        Ok(result)
    }

    fn adjust_traf_timestamps(&mut self, traf_data: &[u8]) -> io::Result<Vec<u8>> {
        let mut result = Vec::with_capacity(traf_data.len());
        let mut offset = 8;

        result.extend_from_slice(&traf_data[0..8]);

        while offset < traf_data.len() {
            if offset + 8 > traf_data.len() {
                break;
            }

            let (box_type, box_size) = self.parse_box_header(&traf_data[offset..])?;

            if offset + box_size > traf_data.len() {
                break;
            }

            let box_data = &traf_data[offset..offset + box_size];

            match &box_type {
                b"tfdt" => {
                    // baseMediaDecodeTime 조정
                    let adjusted_tfdt = self.adjust_tfdt(box_data)?;
                    result.extend_from_slice(&adjusted_tfdt);
                }
                _ => {
                    result.extend_from_slice(box_data);
                }
            }

            offset += box_size;
        }

        Ok(result)
    }

    fn adjust_tfdt(&mut self, tfdt_data: &[u8]) -> io::Result<Vec<u8>> {
        if tfdt_data.len() < 16 {
            return Ok(tfdt_data.to_vec());
        }

        let version = tfdt_data[8];
        let mut result = tfdt_data.to_vec();

        if version == 1 {
            // 64-bit baseMediaDecodeTime
            if tfdt_data.len() < 20 {
                return Ok(tfdt_data.to_vec());
            }

            let decode_time = u64::from_be_bytes([
                tfdt_data[12],
                tfdt_data[13],
                tfdt_data[14],
                tfdt_data[15],
                tfdt_data[16],
                tfdt_data[17],
                tfdt_data[18],
                tfdt_data[19],
            ]);

            // 첫 세그먼트의 타임스탬프를 base로 설정
            if self.base_decode_time.is_none() {
                self.base_decode_time = Some(decode_time);
            }

            // 타임스탬프 조정
            let adjusted_time = decode_time.saturating_sub(self.base_decode_time.unwrap());
            let time_bytes = adjusted_time.to_be_bytes();

            result[12..20].copy_from_slice(&time_bytes);
        } else {
            // 32-bit baseMediaDecodeTime
            let decode_time =
                u32::from_be_bytes([tfdt_data[12], tfdt_data[13], tfdt_data[14], tfdt_data[15]])
                    as u64;

            if self.base_decode_time.is_none() {
                self.base_decode_time = Some(decode_time);
            }

            let adjusted_time = decode_time.saturating_sub(self.base_decode_time.unwrap());
            let time_bytes = (adjusted_time as u32).to_be_bytes();

            result[12..16].copy_from_slice(&time_bytes);
        }

        Ok(result)
    }

    /// 현재 base decode time 반환 (디버깅용)
    pub fn get_base_decode_time(&self) -> Option<u64> {
        self.base_decode_time
    }

    /// 프로세서 리셋
    pub fn reset(&mut self) {
        self.base_decode_time = None;
    }
}

impl Default for FragmentedMP4Processor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = FragmentedMP4Processor::new();
        assert!(processor.init_segment.is_empty());
        assert!(processor.base_decode_time.is_none());
    }
}
