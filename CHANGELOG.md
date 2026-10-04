# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2026-10-04

### Added

- Added automated PyPI publication through GitHub Actions and PyPI Trusted Publishing/OIDC.
- Added cross-platform release wheels for Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x86_64, plus a source distribution.
- Added CI validation of built Python distribution artifacts before release tags are created.
- Added declared and tested CPython 3.13 and 3.14 support.

### Changed

- Consolidated tag-driven PyPI and crates.io publication into a single release workflow.
- Kept crates.io publication idempotent: an already-published crate version is detected and skipped instead of failing the release.


## [0.2.0] - 2026-10-04

### Added

- Added `rlib` output alongside the Python extension `cdylib` so downstream Rust/PyO3 crates can reuse `tempoch-py`.
- Added the public `tempoch_py::interop` API for converting timezone-aware Python datetimes to `Time<UTC>` and converting `Time<UTC>` back to aware UTC Python datetimes.
- Added `Period<UTC>` endpoint conversion helpers and explicit `TimeContext` variants for context-dependent conversions.
- Added downstream PyO3 consumer integration coverage for the reusable Rust interop surface.
- Added tag-driven crates.io publication with release validation, dry-run support, and duplicate-version detection.

### Changed

- Updated the Rust dependency to `tempoch` 0.7.x.
- Aligned the bindings and downstream integration surface with PyO3 0.29.x.
- Normalized timezone-aware Python datetimes with non-UTC offsets to UTC before conversion.
- Synchronized the Rust crate and Python distribution metadata at version 0.2.0.

### Fixed

- Reject naive Python datetimes instead of silently assigning timezone semantics.

### Removed

- Removed the stale embedded tempoch submodule in favor of the published `tempoch` crate.

## [0.1.0] - 2026-03-07

### Added

- Initial `tempoch` Python bindings for Julian Date, Modified Julian Date, astronomical time-scale conversion, and time periods.
- Initial Rust-backed Python package, examples, tests, and CI coverage.
