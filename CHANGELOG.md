# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.6] - 2023-05-21
### Fixed
- Parsing a `PERF_SAMPLE_RAW` field in `Sample` now properly handles padding
  bytes when the field size is not a multiple of 8 bytes.
- Parsing a `PERF_SAMPLE_STACK_USER` field in `Sample` will no longer parse the
  `dyn_size` field when the static stack size is `0`.

## [0.1.5] - 2023-05-21
### Fixed
- Parse the header for the `PERF_SAMPLE_RAW` field in `Sample` as a `u32`
  instead of a `u64`.

## [0.1.4] - 2023-10-23
### Changed
- Internal enum types are now declared using the `c-enum` crate.

## [0.1.3] - 2023-10-23
### Added
- `Parse` is now implemented for `perf_event_attr` from `perf_event_open_sys2`.

### Changed
- The debug representation of several record types has been changed to make it
  more readable.

### Fixed
- Fixed an infinite loop with unbounded memory usage in `ParseBufCursor::new`
  when the first chunk returned by the `ParseBuf` had length 0.

## [0.1.2] - 2023-05-16
### Changed
- Fixed compile breakage due to https://github.com/bitflags/bitflags/issues/353

## [0.1.1] - 2023-05-14
### Added
- `ReadValue::from_group_and_entry` for creating a `ReadValue` from a
  `ReadGroup` and a `GroupEntry`.

## [0.1.0] - 2023-05-14
This is the very first release of the `perf-event-data` crate.

[Unreleased]: https://github.com/phantomical/perf-event-data/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/phantomical/perf-event-data/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/phantomical/perf-event-data/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/phantomical/perf-event-data/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/phantomical/perf-event-data/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/phantomical/perf-event-data/releases/tag/v0.1.0
