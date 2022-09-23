# Changelog

## Release 0.27.1

### Changed

- use bitcoin_hashes 0.11 also for build dep

## Release 0.27.0

### Added

- Introduced CHANGELOG
- Supports windows OS
- Provide errors if `rpcuser` and `rpcpassword` are provided

### Changed

- use bitcoin dep to 0.29.1

### Fixed

- fix bitcoin 0.23 on MacOS X
- fix test flakiness

### Removed

- removed `datadir` from `ConnectionParams`, use equivalent `workdir()`
