# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
