# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
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
