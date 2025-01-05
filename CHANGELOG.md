# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->

## [Unreleased]

### Fixed

- Removed log message from stdout.
- Corrected starting node process in Windows.

## [0.1.0] - 2025-01-05

### Added

- `synf init` command would bootstrap initial `synf.toml` file.

## [0.0.1] - 2025-01-01

### Added

- Initial release.
- Command `synf dev [path]` would start MCP dev server.
- `synf dev` supports kotlin, typescript, python and golang.
- `synf dev` watches default and extra configured dirs/files for changes to trigger server restart. 
- `synf dev` sends list_updated for tools/prompts/resources after server restart.

<!-- next-url -->
[Unreleased]: https://github.com/strowk/synf/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/strowk/synf/compare/v0.0.1...v0.1.0
[0.1.0]: https://github.com/strowk/synf/releases/tag/v0.1.0
