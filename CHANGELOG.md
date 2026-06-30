# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-06-30

### Changed

- Updated console output for better clarity and readability, making it clearer which dependencies are being modified.
- Added `-verbose` flag to provide detailed information when updating dependencies with the `--upgrade` flag, allowing users to see the specific changes being made by `uv`.
- Tidied up code.
- Updated all documentation to include installation methods, project links, and detailed information on how the tool works.
- Updated version in `pyproject.toml` to be dynamically retrieved from the `Cargo.toml` file, allowing `Cargo.toml` to be the single source of truth for the version number.

### Removed

- Removed handling of `~=` compatibility operator as `uv` will not resolve to a breaking version, making it unnecessary to skip these constraints.

## [0.1.1] - 2026-06-28

### Added

- Added normalise version method to avoid false positives in version comparison when comparing versions with different formats (e.g., `1.0` vs `1.0.0`).

### Changed

- Updated exit routes to use appropriate exit codes for different error scenarios, ensuring CI compatibility and better error handling.

## [0.1.0] - 2026-06-28

### Added

- Created initial implementation of tool with checking and updating functionality for `pyproject.toml` dependency constraints using versions resolved by `uv`.

[0.1.2]: https://github.com/javidahmed64592/uv-align/releases/tag/v0.1.2
[0.1.1]: https://github.com/javidahmed64592/uv-align/releases/tag/v0.1.1
[0.1.0]: https://github.com/javidahmed64592/uv-align/releases/tag/v0.1.0
[SemVer]: https://semver.org
