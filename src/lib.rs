use std::io;

mod mp4_writer;
mod ts_parser;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn convert_ts_to_mp4_wasm(ts_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    convert_ts_to_mp4(ts_data).map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

pub fn convert_ts_to_mp4(ts_data: &[u8]) -> io::Result<Vec<u8>> {
    // Parse TS packets
    let media_data = ts_parser::parse_ts_packets(ts_data)?;

    // Create MP4 container
    let mp4_data = mp4_writer::create_mp4(media_data)?;

    Ok(mp4_data)
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
