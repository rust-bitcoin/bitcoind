# Changelog

## Release 0.34.0

- upgrade bitcoincore dep to 0.18.0 and with it bitcoin to 0.31.0

## Release 0.28.0

### Changed

- bump `ureq`'s version to `2.5.0`
- bump `flate2`'s version to `1.0.24`
- bump `filetime`'s version to `0.2.18`

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
