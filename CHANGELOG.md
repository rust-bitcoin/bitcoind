# Changelog

## 0.36.0

- Remove range dependencies for `bitcoincore-rpc` and depend on the
  latest version `v0.19.0` [#163](https://github.com/rust-bitcoin/bitcoind/pull/163).

## 0.35.2

- Use range dependencies for `bitcoincore-rpc` and `bitcoin_hashes`

## Release 0.35.1

- Bump MSRV to 1.56.1
- Add `BITCOIND_SKIP_DOWNLOAD` build feature

## Release 0.34.2

- Support Bitcoin Core 26.0

## Release 0.34.1

- Optionally enable ZMQ

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
