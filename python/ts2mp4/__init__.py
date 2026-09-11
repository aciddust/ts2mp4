"""ts2mp4 - MPEG-2 TS 와 fragmented MP4 를 MP4 로 바꾸는 변환기.

모든 함수는 bytes 를 받아 bytes 를 돌려준다. 임시 파일도, 외부 실행 파일도
필요 없다. 잘못된 입력은 ValueError 로 올라온다.

    >>> import ts2mp4
    >>> mp4 = ts2mp4.convert_ts_to_mp4(ts_bytes)
    >>> mp4 = ts2mp4.mux_fmp4_tracks(video_bytes, audio_bytes)
"""

from ._ts2mp4 import (
    FragmentedMP4Processor,
    __version__,
    convert_mp4_reset_timestamps,
    convert_ts_to_mp4,
    defragment_mp4,
    extract_thumbnail_from_mp4,
    extract_thumbnail_from_ts,
    mux_fmp4_tracks,
    reset_mp4_timestamps,
)

__all__ = [
    "FragmentedMP4Processor",
    "__version__",
    "convert_mp4_reset_timestamps",
    "convert_ts_to_mp4",
    "defragment_mp4",
    "extract_thumbnail_from_mp4",
    "extract_thumbnail_from_ts",
    "mux_fmp4_tracks",
    "reset_mp4_timestamps",
]
