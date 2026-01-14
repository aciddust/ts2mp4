# 사용 가이드

## 실행 방법

```bash
# 빌드
cargo build --release

# 변환 실행
./target/release/ts2mp4 input.ts output.mp4

# 확인
ffprobe output.mp4
```

## 재생 테스트

생성된 MP4 파일의 비디오와 오디오정보를 아래의 플레이어에서 확인했습니다.

- **QuickTime Player** (macOS 기본 플레이어)
- **VLC Media Player**
- **FFplay** (ffmpeg)
- 대부분의 웹 브라우저 (HTML5 video)
- Windows Media Player
- 모바일 플레이어 (iOS Safari, Android Chrome 등)

**특이사항**:

- 본 변환기는 QuickTime 호환성을 최우선으로 개발되었습니다.

## 주요 작업내용

### 완료된 내용 요약

- [x] ~~오디오 트랙 지원 (AAC)~~
- [x] ~~비디오/오디오 동기화~~
- [x] ~~QuickTime 호환성~~

### MP4 표준에 맞추려고 노력함

- 비디오 지원
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
- 오디오 지원
  - AAC 프레임 추출 및 muxing
  - smhd (Sound Media Header)
  - esds 박스 (AudioSpecificConfig)
    - AAC-LC profile
    - 48kHz 샘플레이트
    - Stereo 채널 설정
- 비디오/오디오 동기화 (PTS 기반)

### 메타데이터 추출

- SPS (Sequence Parameter Set) 파싱
- PPS (Picture Parameter Set) 추출
- 해상도 자동 감지
- avcC 박스 생성 (H.264 디코더 설정)

### 샘플 테이블 호환

- 각 프레임의 크기 기록 (STSZ)
- 정확한 청크 오프셋 계산 (STCO)
- 시간 정보 포함 (STTS)
- 단일 청크 최적화 (호환성)

### Timescale 통일

- 모든 트랙에서 90kHz timescale 사용
- 비디오: sample_delta = 3000 (30fps)
- 오디오: sample_delta = 1920 (AAC 1024 samples @ 48kHz)
- QuickTime 플레이어 실행을 기준으로 작성함
- avcC 박스 생성 (디코더 설정)

## 디버깅 및 검증

자세한 검증 방법은 [DEV_GUIDE.md](DEV_GUIDE.md)를 참고하세요.

### 빠른 검증

```bash
# 기본 정보
ffprobe output.mp4

# 재생 테스트
ffplay output.mp4
```

### 예상 출력 예시

```bash
Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'output2.mp4':
  Duration: 00:00:23.36
  Stream #0:0: Video: h264 (Main), yuv420p, 1280x720, 3203 kb/s, 30 fps
  Stream #0:1: Audio: aac (LC), 48000 Hz, stereo, fltp, 192 kb/s
```

### 제한사항

현재 버전의 제한사항:

1. **비디오만 처리**: 오디오 트랙은 아직 지원하지 않음
2. **H.264 전용**: MPEG-2나 다른 코덱은 미지원
3. **단순한 SPS 파싱**: 복잡한 프로파일은 기본값 사용
4. **고정 프레임레이트**: 가변 프레임레이트 미지원
