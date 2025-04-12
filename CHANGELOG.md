# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

<!-- next-header -->

## [Unreleased]

WIP

## [0.2.3] - 2025-04-12

- bump: crossbeam channel dependency to 0.5.15.

## [0.2.2] - 2025-03-30

### Fixed

- Fixed when configuration in `synf.toml` was ignored for run command, by @ezyang.

## [0.2.1] - 2025-01-08

### Fixed

- Correctly reading configuration for default_paths and extra_paths.
- Correctly reading configuration for run command.

## [0.2.0] - 2025-01-07

### Changed

- Build command output is not shown in stderr anymore.

### Fixed

- Sometimes `synf dev` would get stuck on Windows if using powershell for build command.

### Added

- `synf dev` now supports tracking and resending resource subscriptions after server restart.
- `synf init` now adds comment about resend_resource_subscriptions to `synf.toml` file.

## [0.1.1] - 2025-01-05

### Fixed

- Removed log message from stdout.
- Corrected starting node process in Windows.
- Removed extra endline from initialize output.

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
[Unreleased]: https://github.com/strowk/synf/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/strowk/synf/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/strowk/synf/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/strowk/synf/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/strowk/synf/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/strowk/synf/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/strowk/synf/compare/v0.0.1...v0.1.0
[0.1.0]: https://github.com/strowk/synf/releases/tag/v0.1.0
