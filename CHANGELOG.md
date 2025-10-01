# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2025-10-01

### Changed
- Bump Bevy version to 0.17.0
- Updated all dependencies to Bevy 0.17.0 compatible versions

### Fixed
- Fixed untyped observers not firing properly
- Updated HttpClientPlugin doctest to use MinimalPlugins
- Resolved GitHub CI/CD workflow issues

### Added
- Added RUSTFLAGS for WASM example builds to improve compilation
- Added flag for WASM example execution

### Development
- Fixed import ordering to pass cargo fmt checks
- Improved overall code formatting and consistency

## [0.8.3] - 2025-06-25

### Added
- Safe HTTP client builder methods (`try_build()`, `try_with_type()`) for better error handling
- New error types: `HttpClientBuilderError` and `JsonSerializationError`
- JSON serialization with fallback strategies (`json_with_fallback()`, `json_safe()`)
- Comprehensive GitHub Actions CI/CD pipeline
  - Automated testing including doctests, examples, and WASM compilation
  - Automatic GitHub Pages deployment with live WASM demo
  - Automated releases to GitHub and crates.io on tag creation
- Complete documentation test coverage (all doctests now pass)
- Enhanced error logging using `bevy_log` throughout the codebase

### Changed
- Made `bevy_log` a direct dependency (no longer optional feature)
- Improved error handling in async HTTP request processing
- Enhanced JSON serialization with safe fallback mechanisms
- Updated all examples to use new safe builder methods
- Modernized GitHub Actions workflows with latest action versions

### Deprecated
- `build()` method - use `try_build()` instead for better error handling
- `with_type()` method - use `try_with_type()` instead for better error handling

### Fixed
- Removed all `unwrap()` calls that could cause runtime panics
- Fixed all failing documentation tests (22 doctests now pass)
- Improved error handling in typed request processing
- Fixed clippy warnings about derivable Default implementations
- Enhanced WASM compatibility and optimization

### Security
- Eliminated potential panic points in async request handling
- Added payload size warnings for large JSON requests (>50MB)
- Improved error propagation and logging for better debugging

## [0.8.2] - 2025-06-03

* access inner value T from a TypedResponse reference [#11](https://github.com/foxzool/bevy_http_client/pull/11)

## [0.8.1]

- add component observe 

## [0.8.0]

- bump bevy version to 0.16.0

## [0.7.0] - 2024-11-30

- bump bevy version to 0.15.0

## [0.6.0] - 2024-07-05

- bump bevy version to 0.14.0

## [0.5.2] - 2024-04-23

* Fix: extract inner type [#8](https://github.com/foxzool/bevy_http_client/issues/8)

## [0.5.1] - 2024-02-28

* Fix: Typed error handling [#6](https://github.com/foxzool/bevy_http_client/pull/6)
* Fix: use crossbeam-channel replace task for wasm error

## [0.5.0] - 2024-02-20

- now it's use event to send request and handle response

## [0.4.0] - 2024-02-19

- bump bevy version to 0.13.0