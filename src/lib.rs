use std::io;

mod fmp4_processor;
mod mp4_parser;
mod mp4_writer;
mod thumbnail;
mod ts_parser;

// Re-export thumbnail functions
pub use thumbnail::{extract_thumbnail_from_mp4, extract_thumbnail_from_ts};

// Re-export MP4 parser functions
pub use mp4_parser::{defragment_mp4, reset_mp4_timestamps};

// Re-export fMP4 processor
pub use fmp4_processor::FragmentedMP4Processor;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn convert_ts_to_mp4_wasm(ts_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    convert_ts_to_mp4(ts_data).map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn convert_ts_to_mp4_reset_timestamps_wasm(ts_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    convert_ts_to_mp4_with_options(ts_data, true)
        .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn reset_mp4_timestamps_wasm(mp4_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    reset_mp4_timestamps(mp4_data)
        .map_err(|e| JsValue::from_str(&format!("Timestamp reset error: {}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn defragment_mp4_wasm(mp4_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    defragment_mp4(mp4_data).map_err(|e| JsValue::from_str(&format!("Defragment error: {}", e)))
}

/// Convert MP4 with timestamp reset (equivalent to CLI: ts2mp4 convert -i input.mp4 -o output.mp4 --reset-timestamps)
/// This function combines defragment and reset operations automatically:
/// 1. If input is fragmented MP4 (fMP4): defragments to regular MP4 (timestamps automatically start from 0)
/// 2. If input is already regular MP4: resets timestamps to start from 0
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn convert_mp4_reset_timestamps_wasm(mp4_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    convert_mp4_reset_timestamps(mp4_data)
        .map_err(|e| JsValue::from_str(&format!("Convert MP4 error: {}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn extract_thumbnail_from_ts_wasm(ts_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    extract_thumbnail_from_ts(ts_data)
        .map_err(|e| JsValue::from_str(&format!("Thumbnail extraction error: {}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn extract_thumbnail_from_mp4_wasm(mp4_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    extract_thumbnail_from_mp4(mp4_data)
        .map_err(|e| JsValue::from_str(&format!("Thumbnail extraction error: {}", e)))
}

/// Fragmented MP4 프로세서 (WASM용)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct FragmentedMP4ProcessorWasm {
    processor: FragmentedMP4Processor,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl FragmentedMP4ProcessorWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            processor: FragmentedMP4Processor::new(),
        }
    }

    /// 초기화 세그먼트 설정 (m4s 파일)
    #[wasm_bindgen]
    pub fn set_init_segment(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.processor
            .set_init_segment(data)
            .map_err(|e| JsValue::from_str(&format!("Init segment error: {}", e)))
    }

    /// 미디어 세그먼트 처리 (m4v 파일)
    #[wasm_bindgen]
    pub fn process_segment(&mut self, data: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.processor
            .process_segment(data)
            .map_err(|e| JsValue::from_str(&format!("Segment processing error: {}", e)))
    }

    /// 프로세서 리셋
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.processor.reset();
    }

    /// 현재 base decode time 반환 (디버깅용)
    #[wasm_bindgen]
    pub fn get_base_decode_time(&self) -> Option<f64> {
        self.processor.get_base_decode_time().map(|t| t as f64)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

pub fn convert_ts_to_mp4(ts_data: &[u8]) -> io::Result<Vec<u8>> {
    convert_ts_to_mp4_with_options(ts_data, false)
}

pub fn convert_ts_to_mp4_with_options(
    ts_data: &[u8],
    reset_timestamps: bool,
) -> io::Result<Vec<u8>> {
    // Parse TS packets
    let media_data = ts_parser::parse_ts_packets(ts_data)?;

    // Create MP4 container
    let mp4_data = mp4_writer::create_mp4_with_options(media_data, reset_timestamps)?;

    Ok(mp4_data)
}

/// Convert MP4 with timestamp reset (equivalent to CLI: ts2mp4 convert --reset-timestamps)
/// This function replicates the exact behavior of the CLI convert command:
/// 1. If input is fragmented MP4 (fMP4): defragments to regular MP4 (timestamps automatically start from 0)
/// 2. If input is already regular MP4: resets timestamps to start from 0
pub fn convert_mp4_reset_timestamps(mp4_data: &[u8]) -> io::Result<Vec<u8>> {
    // Try to defragment first (this automatically resets timestamps)
    match defragment_mp4(mp4_data) {
        Ok(data) => {
            // Successfully defragmented - timestamps are already reset to 0
            Ok(data)
        }
        Err(_) => {
            // Not a fragmented MP4, or defragmentation failed
            // Apply timestamp reset to regular MP4
            reset_mp4_timestamps(mp4_data)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        // This is a placeholder test
        // In practice, you'd need sample TS data
        let ts_data = vec![0x47; 188]; // Mock TS packet
        let result = convert_ts_to_mp4(&ts_data);

        // Should fail with empty/invalid data
        assert!(result.is_err());
    }
}
