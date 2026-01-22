# TS2MP4 Web 사용 가이드

## 🚀 빠른 시작

### 1. WASM 빌드하기

```bash
# wasm-pack 설치 (처음 한 번만)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# WASM 빌드
wasm-pack build --target web --out-dir pkg
```

빌드가 완료되면 `pkg/` 디렉토리에 다음 파일들이 생성됩니다:
- `ts2mp4.js` - JavaScript 바인딩
- `ts2mp4_bg.wasm` - 컴파일된 WASM 바이너리
- `ts2mp4.d.ts` - TypeScript 타입 정의

### 2. 로컬 서버 실행

```bash
# Python 3
python3 -m http.server 8000

# 또는 Node.js
npx http-server -p 8000
```

브라우저에서 열기:
- 기본 변환기: http://localhost:8000/web/
- Reset Timestamps 예제: http://localhost:8000/web/example-reset-timestamps.html

## 📖 사용 가능한 WASM 함수

### 1. TS → MP4 변환 (기본)
```javascript
import init, { convert_ts_to_mp4_wasm } from './ts2mp4.js';

await init();
const tsData = new Uint8Array(await file.arrayBuffer());
const mp4Data = convert_ts_to_mp4_wasm(tsData);
```

**CLI 대응:**
```bash
./target/release/ts2mp4 convert -i input.ts -o output.mp4
```

### 2. TS → MP4 변환 (타임스탬프 리셋) ⭐
```javascript
import init, { convert_ts_to_mp4_reset_timestamps_wasm } from './ts2mp4.js';

await init();
const tsData = new Uint8Array(await file.arrayBuffer());
const mp4Data = convert_ts_to_mp4_reset_timestamps_wasm(tsData);
```

**CLI 대응:**
```bash
./target/release/ts2mp4 convert -i input.ts -o output.mp4 --reset-timestamps
```

### 3. MP4 타임스탬프 리셋
```javascript
import init, { reset_mp4_timestamps_wasm } from './ts2mp4.js';

await init();
const mp4Data = new Uint8Array(await file.arrayBuffer());
const resetMp4Data = reset_mp4_timestamps_wasm(mp4Data);
```

**CLI 대응:**
```bash
./target/release/ts2mp4 convert -i input.mp4 -o output.mp4 --reset-timestamps
```

### 4. 썸네일 추출 (TS)
```javascript
import init, { extract_thumbnail_from_ts_wasm } from './ts2mp4.js';

await init();
const tsData = new Uint8Array(await file.arrayBuffer());
const thumbnailH264Data = extract_thumbnail_from_ts_wasm(tsData);
// H.264 NAL 데이터 반환 (WebCodecs API로 디코딩 필요)
```

### 5. 썸네일 추출 (MP4)
```javascript
import init, { extract_thumbnail_from_mp4_wasm } from './ts2mp4.js';

await init();
const mp4Data = new Uint8Array(await file.arrayBuffer());
const thumbnailH264Data = extract_thumbnail_from_mp4_wasm(mp4Data);
```

### 6. Panic Hook 초기화
```javascript
import init, { init_panic_hook } from './ts2mp4.js';

await init();
init_panic_hook(); // Rust panic을 JavaScript 에러로 변환
```

## 🎯 예제 코드

### 완전한 예제

```javascript
import init, {
  convert_ts_to_mp4_reset_timestamps_wasm,
  init_panic_hook
} from './ts2mp4.js';

async function convertTStoMP4WithResetTimestamps(file) {
  try {
    // 1. WASM 초기화
    await init();
    init_panic_hook();
    
    // 2. 파일 읽기
    const arrayBuffer = await file.arrayBuffer();
    const tsData = new Uint8Array(arrayBuffer);
    
    // 3. 변환 (타임스탬프 리셋)
    const mp4Data = convert_ts_to_mp4_reset_timestamps_wasm(tsData);
    
    // 4. 다운로드
    const blob = new Blob([mp4Data], { type: 'video/mp4' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = file.name.replace(/\.ts$/i, '_reset.mp4');
    a.click();
    URL.revokeObjectURL(url);
    
    console.log('✅ 변환 완료!');
  } catch (error) {
    console.error('❌ 변환 실패:', error);
  }
}

// 사용 예
document.getElementById('fileInput').addEventListener('change', async (e) => {
  const file = e.target.files[0];
  if (file && file.name.endsWith('.ts')) {
    await convertTStoMP4WithResetTimestamps(file);
  }
});
```

## 🛠️ 고급 사용법

### 진행률 표시

```javascript
async function convertWithProgress(file, onProgress) {
  onProgress(0, '파일 읽는 중...');
  const arrayBuffer = await file.arrayBuffer();
  
  onProgress(30, '변환 중...');
  const tsData = new Uint8Array(arrayBuffer);
  const mp4Data = convert_ts_to_mp4_reset_timestamps_wasm(tsData);
  
  onProgress(90, '다운로드 준비 중...');
  // ... 다운로드 로직
  
  onProgress(100, '완료!');
}
```

### 에러 처리

```javascript
try {
  const mp4Data = convert_ts_to_mp4_reset_timestamps_wasm(tsData);
} catch (error) {
  if (error.message.includes('Invalid TS')) {
    console.error('유효하지 않은 TS 파일입니다.');
  } else if (error.message.includes('No video')) {
    console.error('비디오 스트림을 찾을 수 없습니다.');
  } else {
    console.error('알 수 없는 오류:', error);
  }
}
```

### TypeScript 사용

```typescript
import init, {
  convert_ts_to_mp4_reset_timestamps_wasm,
  InitOutput
} from './ts2mp4.js';

let wasmModule: InitOutput | null = null;

async function initWasm(): Promise<void> {
  if (!wasmModule) {
    wasmModule = await init();
  }
}

async function convertFile(file: File): Promise<Uint8Array> {
  await initWasm();
  const buffer = await file.arrayBuffer();
  const data = new Uint8Array(buffer);
  return convert_ts_to_mp4_reset_timestamps_wasm(data);
}
```

## 📝 주의사항

1. **CORS 설정**: WASM 파일을 로드하려면 적절한 CORS 헤더가 필요합니다.
2. **파일 크기**: 브라우저 메모리 제한에 주의하세요.
3. **브라우저 호환성**: 최신 브라우저에서 테스트하세요.
4. **MIME 타입**: 서버가 `.wasm` 파일을 `application/wasm`으로 제공하도록 설정하세요.

## 🔗 추가 자료

- [WASM 빌드 가이드](../docs/ko/DEV_GUIDE.md)
- [GitHub 저장소](https://github.com/aciddust/ts2mp4)
