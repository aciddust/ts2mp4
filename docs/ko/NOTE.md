# NOTE

## 작업중 어려웠던 내용

프로젝트 개발 중 해결한 주요 이슈들:

### 1. 오디오가 영상 중간정도 위치에서 끊기는 문제

- **원인**: STTS sample_delta가 1024 (샘플 개수)로 설정됨
- **해결**: 1920으로 수정 (90kHz timescale 반영)

### 2. ffplay는 오디오정보를 가지고있지만 QuickTime에서 중간 이전에 끊기는 문제

- **원인**: MDHD timescale이 48000으로 설정됨
- **해결**: 90000으로 통일

### 3. ffplay는 오디오정보를 가지고있지만 QuickTime에서 12초에 끊기는 문제 (2번 문제 이후 후속)

- **원인**: TKHD duration이 잘못된 timescale로 계산됨
- **해결**: Movie timescale (90kHz) 기준으로 계산

대부분의 오디오 재생 문제는 **timescale 통일**로 해결되었음
