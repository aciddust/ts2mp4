# TS to MP4 Converter

MPEG-2 TS(Transport Stream) 파일을 MP4 형식으로 변환하는 Rust 라이브러리입니다. WebAssembly로 컴파일하여 웹 브라우저에서도 사용할 수 있습니다.

## 특징

- 순수 Rust 구현 (SharedArrayBuffer 불필요)
- WebAssembly 지원
- 단일 스레드 동작으로 웹 환경에서 안전하게 사용 가능
- Zero-copy 최적화

## 설치

```bash
cargo build --release
```

## 사용법

### CLI 사용

```bash
cargo run --release -- input.ts output.mp4
```

### Rust 라이브러리로 사용

```rust
use ts2mp4::convert_ts_to_mp4;
use std::fs;

fn main() -> std::io::Result<()> {
    let ts_data = fs::read("input.ts")?;
    let mp4_data = convert_ts_to_mp4(&ts_data)?;
    fs::write("output.mp4", mp4_data)?;
    Ok(())
}
```

## WebAssembly 빌드

### 사전 요구사항

```bash
# wasm-pack 설치
cargo install wasm-pack

# 또는 wasm32 타겟 추가
rustup target add wasm32-unknown-unknown
```

### WASM 빌드 방법

#### 1. wasm-pack 사용 (권장)

```bash
wasm-pack build --target web
```

이렇게 하면 `pkg/` 디렉토리에 다음 파일들이 생성됩니다:
- `ts2mp4.js` - JavaScript 바인딩
- `ts2mp4_bg.wasm` - WebAssembly 바이너리
- `ts2mp4.d.ts` - TypeScript 타입 정의

#### 2. 직접 빌드

```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ts2mp4.wasm --out-dir pkg --target web
```

### 웹에서 사용하기

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>TS to MP4 Converter</title>
</head>
<body>
    <input type="file" id="fileInput" accept=".ts">
    <button id="convertBtn">Convert to MP4</button>
    
    <script type="module">
        import init, { convert_ts_to_mp4_wasm } from './pkg/ts2mp4.js';
        
        async function convertFile() {
            // WASM 초기화
            await init();
            
            const fileInput = document.getElementById('fileInput');
            const file = fileInput.files[0];
            
            if (!file) {
                alert('Please select a file');
                return;
            }
            
            // 파일 읽기
            const arrayBuffer = await file.arrayBuffer();
            const tsData = new Uint8Array(arrayBuffer);
            
            try {
                // TS를 MP4로 변환 (SharedArrayBuffer 불필요)
                const mp4Data = convert_ts_to_mp4_wasm(tsData);
                
                // MP4 파일 다운로드
                const blob = new Blob([mp4Data], { type: 'video/mp4' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = 'output.mp4';
                a.click();
                URL.revokeObjectURL(url);
                
                alert('Conversion successful!');
            } catch (error) {
                console.error('Conversion failed:', error);
                alert('Conversion failed: ' + error);
            }
        }
        
        document.getElementById('convertBtn').addEventListener('click', convertFile);
    </script>
</body>
</html>
```

## SharedArrayBuffer를 사용하지 않는 이유

이 라이브러리는 의도적으로 SharedArrayBuffer를 사용하지 않도록 설계되었습니다:

### 장점

1. **브라우저 호환성**: SharedArrayBuffer는 COOP/COEP 헤더 설정이 필요하여 많은 호스팅 환경에서 사용하기 어렵습니다
2. **보안**: Spectre 취약점 완화를 위해 많은 브라우저에서 제한됩니다
3. **단순성**: 복잡한 서버 설정 없이 바로 사용 가능합니다
4. **단일 스레드**: 메모리 관리가 간단하고 예측 가능합니다

### 성능 최적화 방법

SharedArrayBuffer 없이도 좋은 성능을 얻을 수 있습니다:

1. **스트리밍 처리**: 전체 파일을 메모리에 로드하지 않고 청크 단위로 처리
2. **Web Workers**: 메인 스레드 블로킹 방지를 위해 Worker에서 실행
3. **비동기 처리**: 큰 파일의 경우 작업을 나누어 처리

```javascript
// Web Worker에서 사용하는 예시
// worker.js
import init, { convert_ts_to_mp4_wasm } from './pkg/ts2mp4.js';

self.onmessage = async (e) => {
    await init();
    
    try {
        const mp4Data = convert_ts_to_mp4_wasm(e.data);
        self.postMessage({ success: true, data: mp4Data });
    } catch (error) {
        self.postMessage({ success: false, error: error.message });
    }
};

// main.js
const worker = new Worker('worker.js', { type: 'module' });

worker.onmessage = (e) => {
    if (e.data.success) {
        // MP4 데이터 처리
        const blob = new Blob([e.data.data], { type: 'video/mp4' });
        // ...
    } else {
        console.error('Conversion failed:', e.data.error);
    }
};

// 변환 시작
worker.postMessage(tsData);
```

## 제한사항

현재 버전은 기본적인 TS to MP4 변환 기능을 제공합니다. 다음 기능들은 향후 추가될 예정입니다:

- 완전한 MP4 메타데이터 생성 (moov, trak 등)
- 다양한 코덱 지원
- 타임스탬프 처리
- 다중 오디오/자막 트랙 지원

## 라이선스

MIT

## 기여

이슈나 PR은 언제나 환영합니다!
