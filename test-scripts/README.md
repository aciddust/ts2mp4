# Test Scripts

MP4 파일 분석 및 테스트를 위한 유틸리티 스크립트 모음

## 필요한 바이너리

- Python 3.10 <=
- ffmpeg, ffprobe (일부 스크립트)
- xxd (hex dump 도구)

## MP4 박스 분석 도구

### analyze_mp4.py

MP4 파일의 전체 구조를 분석 (stsc, stco, stsz 등 샘플 테이블 분석)

```bash
python test-scripts/analyze_mp4.py output.mp4
```

### analyze_mvhd.py

mvhd(Movie Header) 박스의 timescale, duration, version 등을 상세 분석

```bash
python test-scripts/analyze_mvhd.py file.mp4
```

### check_ftyp.py

ftyp(File Type) 박스의 brand 확인 및 QuickTime 호환성 체크

```bash
python test-scripts/check_ftyp.py file.mp4
```

### check_edts.py

edts(Edit Box) 박스 존재 여부 및 내용 확인

```bash
python test-scripts/check_edts.py file.mp4
```

### check_stts.py

stts(Sample Time-to-Sample) 박스 분석

```bash
python test-scripts/check_stts.py file.mp4
```

## Fragmented MP4 분석 도구

### analyze-fragments.py

Fragmented MP4의 moof/mdat 구조 분석

```bash
python test-scripts/analyze-fragments.py file.mp4
```

### check-mehd.py

mehd(Movie Extends Header) 박스 확인 및 fragment duration 분석

```bash
python test-scripts/check-mehd.py input-5mb.mp4 fixed-5mb.mp4
```

### check-tfdt.py

tfdt(Track Fragment Decode Time) 박스의 baseMediaDecodeTime 분석

```bash
python test-scripts/check-tfdt.py input-5mb.mp4 fixed-5mb.mp4
```

### check-trun.py

trun(Track Fragment Run) 박스의 sample 정보 분석

```bash
python test-scripts/check-trun.py input-5mb.mp4 fixed-5mb.mp4
```

##  Duration 및 타임스탬프 검증

### check_duration.py

MP4 파일의 duration 확인 (mvhd, tkhd, mdhd)

```bash
python test-scripts/check_duration.py output.mp4
```

### check_all_durations.py

모든 트랙의 duration을 상세 확인

```bash
python test-scripts/check_all_durations.py output.mp4
```

### find-track-duration.py

각 트랙의 tkhd 및 mdhd에서 duration 추출 및 비교

```bash
python test-scripts/find-track-duration.py input-5mb.mp4 fixed-5mb.mp4
```

## 데이터 검증 도구

### check_data_offset.py

moof/traf/trun의 data_offset 값 확인

```bash
python test-scripts/check_data_offset.py file.mp4
```

### check_audio_position.py

오디오 샘플의 위치 및 offset 검증

```bash
python test-scripts/check_audio_position.py output.mp4
```

### verify_audio_data.py

오디오 데이터의 무결성 검증

```bash
python test-scripts/verify_audio_data.py output.mp4
```

### verify_frames.py

ffmpeg로 모든 프레임 디코딩 검증

```bash
python test-scripts/verify_frames.py file.mp4
```

## 비교 도구

### compare_mp4_boxes.py

두 MP4 파일의 박스 구조를 hex로 비교

```bash
python test-scripts/compare_mp4_boxes.py file1.mp4 file2.mp4
```

### compare_streams.py

두 파일의 ffprobe 스트림 정보(코덱, duration, 해상도 등) 비교

```bash
python test-scripts/compare_streams.py file1.mp4 file2.mp4
```

## 통합 테스트

### test_defragment.py

Fragmented MP4 → 일반 MP4 변환의 전체 프로세스를 자동 테스트

- ftyp 확인
- mvhd 분석
- defragment 실행
- 스트림 비교
- 프레임 검증

```bash
python test-scripts/test_defragment.py input.mp4 output.mp4
```

## 디버깅 도구

### debug_n_sec.py

특정 초(n초)의 프레임 데이터를 상세 분석

```bash
python test-scripts/debug_n_sec.py output.mp4
```

### check-v2-duration.py

version 1 mvhd 박스의 duration 확인 (64비트)

```bash
python test-scripts/check-v2-duration.py fixed-5mb-v2.mp4
```
