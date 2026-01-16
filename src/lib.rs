use std::io;

mod frame_extractor;
mod gif_encoder;
mod mp4_writer;
mod thumbnail;
mod ts_parser;

// Re-export functions
pub use gif_encoder::{encode_gif, GifOptions};
pub use thumbnail::{extract_thumbnail_from_mp4, extract_thumbnail_from_ts};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn convert_ts_to_mp4_wasm(ts_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    convert_ts_to_mp4(ts_data).map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn convert_ts_to_gif_wasm(ts_data: &[u8], fps: Option<u16>) -> Result<Vec<u8>, JsValue> {
    let options = fps.map(|f| GifOptions {
        fps: f,
        ..Default::default()
    });
    convert_ts_to_gif(ts_data, options)
        .map_err(|e| JsValue::from_str(&format!("GIF conversion error: {}", e)))
}

pub fn convert_ts_to_mp4(ts_data: &[u8]) -> io::Result<Vec<u8>> {
    // Parse TS packets
    let media_data = ts_parser::parse_ts_packets(ts_data)?;

    // Create MP4 container
    let mp4_data = mp4_writer::create_mp4(media_data)?;

    Ok(mp4_data)
}

/// TS 파일을 GIF로 변환
pub fn convert_ts_to_gif(ts_data: &[u8], options: Option<GifOptions>) -> io::Result<Vec<u8>> {
    // Parse TS packets
    let media_data = ts_parser::parse_ts_packets(ts_data)?;

    // 플레이스홀더 프레임 생성 (실제로는 H.264 디코더 필요)
    // TODO: 실제 H.264 디코딩 구현
    let frames = frame_extractor::create_placeholder_frames(&media_data, 10)?;

    // GIF로 인코딩
    let gif_options = options.unwrap_or_default();
    let gif_data = encode_gif(&frames, &gif_options)?;

    Ok(gif_data)
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
