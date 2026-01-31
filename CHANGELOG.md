# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
