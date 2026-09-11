"""ts2mp4 네이티브 확장 모듈의 타입 정의."""

__version__: str

def convert_ts_to_mp4(data: bytes, reset_timestamps: bool = False) -> bytes:
    """MPEG-TS 를 MP4 로 변환한다.

    reset_timestamps 를 켜면 타임스탬프가 0 부터 시작하도록 맞춘다.
    """

def defragment_mp4(data: bytes) -> bytes:
    """fragmented MP4 를 일반 MP4 로 펼친다.

    입력에 moof 가 없으면 ValueError 를 낸다.
    """

def reset_mp4_timestamps(data: bytes) -> bytes:
    """일반 MP4 의 타임스탬프를 0 부터 시작하도록 되돌린다."""

def convert_mp4_reset_timestamps(data: bytes) -> bytes:
    """fMP4 면 펼치고, 일반 MP4 면 타임스탬프만 되돌린다."""

def mux_fmp4_tracks(video: bytes, audio: bytes) -> bytes:
    """따로 전송된 영상 fMP4 와 소리 fMP4 를 트랙 두 개짜리 MP4 로 합친다.

    두 인자 모두 초기화 세그먼트 뒤에 미디어 세그먼트들을 이어붙인 바이트다.
    track_id 가 겹치면 ValueError 를 낸다.
    """

def extract_thumbnail_from_ts(data: bytes) -> bytes:
    """TS 의 첫 키프레임을 H.264 로 뽑는다."""

def extract_thumbnail_from_mp4(data: bytes) -> bytes:
    """MP4 의 첫 키프레임을 H.264 로 뽑는다."""

class FragmentedMP4Processor:
    """세그먼트를 이어서 받아 처리하는 fMP4 처리기."""

    def __init__(self) -> None: ...
    def set_init_segment(self, data: bytes) -> None: ...
    def process_segment(self, data: bytes) -> bytes: ...
    def reset(self) -> None: ...
    @property
    def base_decode_time(self) -> int | None: ...
