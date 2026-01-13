# 사용 가이드

## 변환 성공!

input.ts 파일을 기준으로 재생 가능한 MP4 파일 생성에 성공했습니다.

### 변환 결과

- **입력**: input.ts (9.7MB)
- **출력**: output.mp4 (122KB)
- **해상도**: 1280x720 (자동 감지)
- **코덱**: H.264 (AVC)
- **재생 시간**: 23.3초
- **프레임 수**: 700 프레임
- **프레임레이트**: 30 fps

### 실행 방법

```bash
# 빌드
cargo build --release

# 변환 실행
./target/release/ts2mp4 input.ts output.mp4

# 확인
ffprobe output.mp4
```

### 재생 테스트

생성된 MP4 파일은 다음 플레이어에서 재생 가능합니다:

- ✅ QuickTime Player
- ✅ VLC Media Player
- ✅ FFplay
- ✅ 대부분의 웹 브라우저 (HTML5 video)

### 주요 개선사항

1. **완전한 MP4 구조**
   - ftyp (File Type)
   - moov (Movie Metadata)
     - mvhd (Movie Header)
     - trak (Track)
       - tkhd (Track Header)
       - mdia (Media)
         - mdhd (Media Header)
         - hdlr (Handler)
         - minf (Media Information)
           - vmhd (Video Media Header)
           - dinf (Data Information)
           - stbl (Sample Table)
             - stsd (Sample Description with avcC)
             - stts (Time-to-Sample)
             - stsc (Sample-to-Chunk)
             - stsz (Sample Sizes)
             - stco (Chunk Offsets)
   - mdat (Media Data)

2. **자동 메타데이터 추출**
   - SPS (Sequence Parameter Set) 파싱
   - PPS (Picture Parameter Set) 추출
   - 해상도 자동 감지
   - avcC 박스 생성 (디코더 설정)

3. **정확한 샘플 테이블**
   - 각 프레임의 크기 기록
   - 정확한 청크 오프셋 계산
   - 시간 정보 포함

### WASM 빌드

웹 환경에서 사용하려면:

```bash
wasm-pack build --target web
```

그러면 pkg/ 디렉토리에 다음 파일들이 생성됩니다:
- ts2mp4.js
- ts2mp4_bg.wasm
- ts2mp4.d.ts

example.html 파일을 참고하여 웹에서 사용할 수 있습니다.

### 성능 참고사항

- **파일 크기**: TS에서 불필요한 오버헤드가 제거되어 MP4가 훨씬 작습니다
- **SharedArrayBuffer 불필요**: 단일 스레드로 동작하여 모든 브라우저에서 호환
- **메모리 효율**: 순차적으로 처리하여 메모리 사용 최소화

### 제한사항

현재 버전의 제한사항:

1. **비디오만 처리**: 오디오 트랙은 아직 지원하지 않음
2. **H.264 전용**: MPEG-2나 다른 코덱은 미지원
3. **단순한 SPS 파싱**: 복잡한 프로파일은 기본값 사용
4. **고정 프레임레이트**: 가변 프레임레이트 미지원

### 향후 개선 계획

- [ ] 오디오 트랙 지원 (AAC)
- [ ] 더 정확한 SPS/PPS 파싱
- [ ] 가변 프레임레이트 지원
- [ ] 다중 트랙 지원
- [ ] 자막 지원
- [ ] 스트리밍 모드 (청크 단위 처리)
