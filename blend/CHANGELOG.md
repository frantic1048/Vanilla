# Changelog

All notable changes to blend will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.11](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.10...blend-v0.2.11) - 2026-07-02

### Other
- Improve blend CLI help discoverability
- Keep read commands from updating blend state

## [0.2.10](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.9...blend-v0.2.10) - 2026-06-25

### Added
- *(blend)* contract versioning and migration framework

### Fixed
- *(blend)* store remembered checkout in state file

## [0.2.9](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.8...blend-v0.2.9) - 2026-06-15

### Fixed
- *(blend)* remove non-root user from Docker image

## [0.2.8](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.7...blend-v0.2.8) - 2026-06-15

### Other
- *(blend)* publish Docker image to ghcr.io on release

## [0.2.7](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.6...blend-v0.2.7) - 2026-06-11

### Added
- *(blend)* add check and format commands

## [0.2.6](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.5...blend-v0.2.6) - 2026-06-09

### Other
- Add basic process sandbox for blend ([#15](https://github.com/frantic1048/Vanilla/pull/15))

## [0.2.5](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.4...blend-v0.2.5) - 2026-05-26

### Added
- replace chflags/chattr with native API call

### Other
- switch nickel from git tag to crate release

## [0.2.4](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.3...blend-v0.2.4) - 2026-05-22

### Added
- update blend-dir config after operating on a valid blend dir

## [0.2.3](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.2...blend-v0.2.3) - 2026-05-19

### Fixed
- avoid unnecessary immutable flag mutation; avoid walking outside blend's ownership boundary

## [0.2.2](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.1...blend-v0.2.2) - 2026-05-14

### Added
- refine table layout

### Fixed
- *(blend)* fallback to config when no ./oroder dir exist

### Other
- fix initial changelog and refine changelog generation

## [0.2.1](https://github.com/frantic1048/Vanilla/compare/blend-v0.2.0...blend-v0.2.1) - 2026-05-13

### Other

- update Cargo.lock dependencies

## [0.2.0](https://github.com/frantic1048/Vanilla/releases/tag/blend-v0.2.0) - 2026-05-13

### Added

- brand new blend

### Fixed

- *(blend)* run release-plz from workspace root

### Other

- *(blend)* add release automation
