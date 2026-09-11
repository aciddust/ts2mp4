# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Python bindings, published as `ts2mp4` on PyPI**
  - Every entry point is exposed as `bytes` in, `bytes` out, so a Python
    service can use the library without temp files or a binary on `PATH`
  - Arguments are copied and the GIL is released during the work
  - `abi3` wheels: one per platform covers Python 3.9 and newer, instead of
    one per Python minor version
  - Gated behind an optional `python` feature, so default builds, the CLI and
    the WebAssembly target are unchanged. The WASM module exports exactly the
    same functions as before
- **`mux_fmp4_tracks`: combine separately delivered fMP4 video and audio into one MP4**
  - HLS can ship picture and sound as independent streams (`EXT-X-MEDIA`),
    each an fMP4 with its own `moov`. `defragment_mp4` takes a single input,
    so it cannot join them and the sound is lost
  - New entry point accepts two byte streams (init segment followed by its
    media segments) and emits a regular MP4 carrying both tracks
  - Reuses the existing fragment reader and `trak` builder; `mvex` is dropped
    from the muxed output, and duplicate `track_id`s are rejected rather than
    silently renumbered
  - No existing function was modified: `defragment_mp4`,
    `convert_ts_to_mp4*`, `reset_mp4_timestamps`, the thumbnail helpers and
    `FragmentedMP4Processor` all produce byte-identical output

## [0.3.2] - 2026-06-06

### Fixed

- **Video sample duration now derived from PTS instead of hardcoded 30fps**
  - `mp4_writer` previously hardcoded every video sample to 3000 ticks
    (90kHz / 30fps) for `mvhd`/`tkhd`/`mdhd`/`stts`, causing 60fps TS
    sources to be written at half speed (video track ran 2x the audio
    duration)
  - Now computes the per-sample delta from the actual PTS range
    (`(max - min) / (n - 1)`, assuming CFR) with a 3000-tick fallback
  - Added unit tests for 60fps, 30fps, reordered PTS, and the fallback path

## [0.3.1] - 2026-01-31

### Fixed

- **Reverted to v0.1.2 legacy timing behavior**
  - Simplified MP4 timestamp handling by removing complex timestamp analysis
  - Uses fixed 30fps timing assumption for predictable and stable output
  - `reset_timestamps` parameter now ignored for API compatibility
  - Removed 189 lines of dynamic duration calculation logic
  - Better suited for fixed framerate clipper applications

### Changed

- MP4 writer now uses simplified timing model (sample_count × 3000 @ 90kHz)
- STTS boxes use single entry with fixed delta instead of dynamic entries

## [0.3.0] - 2026-01-31

### Added

- **Fragmented MP4 (fMP4) streaming processor** (`FragmentedMP4Processor`)
  - Real-time processing of initialization segments (m4s) and media segments (m4v)
  - Automatic timestamp adjustment for continuous playback
  - Support for HLS/DASH streaming workflows
- **MP4 timestamp reset functionality**
  - `defragment_mp4()` - Convert fragmented MP4 to regular MP4 with automatic timestamp reset
  - `reset_mp4_timestamps()` - Reset timestamps of regular MP4 files to start from 0
  - `convert_mp4_reset_timestamps()` - Unified function that handles both fragmented and regular MP4s
- **WebAssembly interfaces for new features**
  - `convert_mp4_reset_timestamps_wasm()` - Convert MP4 with timestamp reset
  - `defragment_mp4_wasm()` - Defragment MP4
  - `reset_mp4_timestamps_wasm()` - Reset MP4 timestamps
  - `FragmentedMP4ProcessorWasm` - WASM wrapper for fMP4 streaming processor
- **CLI enhancements**
  - `--reset-timestamps` flag for convert command
  - Automatic detection and handling of fragmented vs regular MP4 files

### Changed

- Updated CLI convert command to support timestamp reset operations
- Enhanced MP4 parser to handle both fragmented and regular MP4 formats

## [0.2.0]

### Added

- Thumbnail extraction from TS files (`extract_thumbnail_from_ts`)
- Thumbnail extraction from MP4 files (`extract_thumbnail_from_mp4`)
- New CLI commands: `thumbnail-ts` and `thumbnail-mp4`
- WebAssembly support for thumbnail extraction
- Example program for thumbnail extraction
- Web demo page for thumbnail extraction (web/thumbnail.html)

### Changed

- CLI now requires a command argument (`convert`, `thumbnail-ts`, or `thumbnail-mp4`)
- Updated documentation to reflect new thumbnail extraction features

## [0.1.1] - Previous Release

### Features

- H.264 video + AAC audio conversion
- TS to MP4 conversion
- WebAssembly support for browser usage
- Pure Rust implementation without SharedArrayBuffer
