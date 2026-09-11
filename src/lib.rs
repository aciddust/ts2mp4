use std::io;

mod fmp4_processor;
mod mp4_parser;
mod mp4_writer;
mod thumbnail;
mod ts_parser;

// Re-export thumbnail functions
pub use thumbnail::{extract_thumbnail_from_mp4, extract_thumbnail_from_ts};

// Re-export MP4 parser functions
pub use mp4_parser::{defragment_mp4, mux_fmp4_tracks, reset_mp4_timestamps};

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

// ---------------------------------------------------------------------------
// 파이썬 바인딩
//
// wasm32 쪽 래퍼와 같은 자리, 같은 방식이다. 네이티브 함수를 감싸기만 하고
// 변환 로직은 하나도 두지 않는다.
//
// 입력은 복사한 뒤 py.detach 로 GIL 을 놓고 처리한다. 미디어 한 편을 다루는 동안 다른
// 파이썬 스레드가 멈추면 서버에서 쓰기 어렵기 때문이다. 복사 비용은 파싱과
// 재조립에 비하면 무시할 수준이다.
// ---------------------------------------------------------------------------
#[cfg(feature = "python")]
mod python {
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use pyo3::types::PyBytes;

    fn to_py(result: std::io::Result<Vec<u8>>) -> PyResult<Vec<u8>> {
        result.map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// MPEG-TS 를 MP4 로 변환한다.
    #[pyfunction]
    #[pyo3(name = "convert_ts_to_mp4", signature = (data, reset_timestamps = false))]
    fn convert_ts_to_mp4<'py>(
        py: Python<'py>,
        data: &[u8],
        reset_timestamps: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let owned = data.to_vec();
        let out = py.detach(move || {
            super::convert_ts_to_mp4_with_options(&owned, reset_timestamps)
        });
        Ok(PyBytes::new(py, &to_py(out)?))
    }

    /// fragmented MP4 를 일반 MP4 로 펼친다.
    #[pyfunction]
    #[pyo3(name = "defragment_mp4")]
    fn defragment_mp4<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
        let owned = data.to_vec();
        let out = py.detach(move || super::defragment_mp4(&owned));
        Ok(PyBytes::new(py, &to_py(out)?))
    }

    /// MP4 타임스탬프를 0 부터 시작하도록 되돌린다.
    #[pyfunction]
    #[pyo3(name = "reset_mp4_timestamps")]
    fn reset_mp4_timestamps<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
        let owned = data.to_vec();
        let out = py.detach(move || super::reset_mp4_timestamps(&owned));
        Ok(PyBytes::new(py, &to_py(out)?))
    }

    /// fMP4 면 펼치고, 일반 MP4 면 타임스탬프만 되돌린다.
    #[pyfunction]
    #[pyo3(name = "convert_mp4_reset_timestamps")]
    fn convert_mp4_reset_timestamps<'py>(
        py: Python<'py>,
        data: &[u8],
    ) -> PyResult<Bound<'py, PyBytes>> {
        let owned = data.to_vec();
        let out = py.detach(move || super::convert_mp4_reset_timestamps(&owned));
        Ok(PyBytes::new(py, &to_py(out)?))
    }

    /// 따로 전송된 영상 fMP4 와 소리 fMP4 를 트랙 두 개짜리 MP4 로 합친다.
    #[pyfunction]
    #[pyo3(name = "mux_fmp4_tracks")]
    fn mux_fmp4_tracks<'py>(
        py: Python<'py>,
        video: &[u8],
        audio: &[u8],
    ) -> PyResult<Bound<'py, PyBytes>> {
        let (v, a) = (video.to_vec(), audio.to_vec());
        let out = py.detach(move || super::mux_fmp4_tracks(&v, &a));
        Ok(PyBytes::new(py, &to_py(out)?))
    }

    /// TS 의 첫 키프레임을 뽑는다.
    #[pyfunction]
    #[pyo3(name = "extract_thumbnail_from_ts")]
    fn extract_thumbnail_from_ts<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
        let owned = data.to_vec();
        let out = py.detach(move || super::extract_thumbnail_from_ts(&owned));
        Ok(PyBytes::new(py, &to_py(out)?))
    }

    /// MP4 의 첫 키프레임을 뽑는다.
    #[pyfunction]
    #[pyo3(name = "extract_thumbnail_from_mp4")]
    fn extract_thumbnail_from_mp4<'py>(
        py: Python<'py>,
        data: &[u8],
    ) -> PyResult<Bound<'py, PyBytes>> {
        let owned = data.to_vec();
        let out = py.detach(move || super::extract_thumbnail_from_mp4(&owned));
        Ok(PyBytes::new(py, &to_py(out)?))
    }

    /// 세그먼트를 이어서 받아 처리하는 fMP4 처리기.
    #[pyclass(name = "FragmentedMP4Processor")]
    struct Processor {
        inner: super::FragmentedMP4Processor,
    }

    #[pymethods]
    impl Processor {
        #[new]
        fn new() -> Self {
            Self {
                inner: super::FragmentedMP4Processor::new(),
            }
        }

        fn set_init_segment(&mut self, data: &[u8]) -> PyResult<()> {
            self.inner
                .set_init_segment(data)
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }

        fn process_segment<'py>(
            &mut self,
            py: Python<'py>,
            data: &[u8],
        ) -> PyResult<Bound<'py, PyBytes>> {
            let out = self
                .inner
                .process_segment(data)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Ok(PyBytes::new(py, &out))
        }

        fn reset(&mut self) {
            self.inner.reset();
        }

        #[getter]
        fn base_decode_time(&self) -> Option<u64> {
            self.inner.get_base_decode_time()
        }
    }

    #[pymodule]
    fn _ts2mp4(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("__version__", env!("CARGO_PKG_VERSION"))?;
        m.add_function(wrap_pyfunction!(convert_ts_to_mp4, m)?)?;
        m.add_function(wrap_pyfunction!(defragment_mp4, m)?)?;
        m.add_function(wrap_pyfunction!(reset_mp4_timestamps, m)?)?;
        m.add_function(wrap_pyfunction!(convert_mp4_reset_timestamps, m)?)?;
        m.add_function(wrap_pyfunction!(mux_fmp4_tracks, m)?)?;
        m.add_function(wrap_pyfunction!(extract_thumbnail_from_ts, m)?)?;
        m.add_function(wrap_pyfunction!(extract_thumbnail_from_mp4, m)?)?;
        m.add_class::<Processor>()?;
        Ok(())
    }
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
