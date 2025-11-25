# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.0 - 2025-11-24
### Added
- Introduced unified `core.rs` with structured error type (`Error`)
- Added POSIX-style sysexit error mapping and improved error reporting helpers
- Added verbose levels (`-v`, `-vv`, `-vvv`) and `--debug` support across all CLIs
- Added structured tracing integration via `asimov_module::tracing`
- Added centralized logging helpers: `info_user`, `warn_user`, `warn_user_with_error`

### Changed
- Major CLI refactor for:
    - `asimov-image-reader`
    - `asimov-image-viewer`
    - `asimov-image-writer`
- All binaries now use the shared core error/logging system
- Improved stderr messages and tracing output during failures
- Improved pipeline behavior (`--union`) and consistent JSON-LD handling
- Refactored repository structure: moved shared logic into `src/core.rs`
- Updated README with production-grade examples and detailed documentation
- Updated `.asimov/module.yaml` to include all three provided programs

### Fixed
- Inconsistent error handling between reader, viewer, and writer
- Missing or incorrect verbose/debug logs in several code paths
- Tracing not active unless `tracing` feature was explicitly enabled

## 0.0.3 - 2025-11-21
### Added
- `asimov-image-writer`

## 0.0.2 - 2025-11-20
### Added
- `asimov-image-viewer`

## 0.0.1 - 2025-11-09
