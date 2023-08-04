[![MIT license](https://img.shields.io/github/license/RCasatta/bitcoind)](https://github.com/RCasatta/bitcoind/blob/master/LICENSE)
[![Crates](https://img.shields.io/crates/v/bitcoind.svg)](https://crates.io/crates/bitcoind)
[![Docs](https://img.shields.io/badge/docs.rs-bitcoind-green)](https://docs.rs/bitcoind)

# Bitcoind

Utility to run a regtest bitcoind process, useful in integration testing environment.

When the auto-download feature is selected by activating one of the version feature, such as `25_0`
for bitcoin core 25.0, starting a regtest node is as simple as that:

```rust
// the download feature is enabled whenever a specific version is enabled, for example `25_0` or `24_0_1`
#[cfg(feature = "download")]
{
  use bitcoincore_rpc::RpcApi;
  let bitcoind = bitcoind::BitcoinD::from_downloaded().unwrap();
  assert_eq!(0, bitcoind.client.get_blockchain_info().unwrap().blocks);
}
```

The build script will automatically download the bitcoin core version 25.0 from [bitcoin core](https://bitcoincore.org),
verify the hashes and place it in the build directory for this crate. If you wish to download from an 
alternate location, for example locally for CI, use the `BITCOIND_DOWNLOAD_ENDPOINT` env var.

When you don't use the auto-download feature you have the following options:

* have `bitcoind` executable in the `PATH`
* provide the `bitcoind` executable via the `BITCOIND_EXEC` env var

```rust
use bitcoincore_rpc::RpcApi;
if let Ok(exe_path) = bitcoind::exe_path() {
  let bitcoind = bitcoind::BitcoinD::new(exe_path).unwrap();
  assert_eq!(0, bitcoind.client.get_blockchain_info().unwrap().blocks);
}
```

Startup options could be configured via the [`Conf`] struct using [`BitcoinD::with_conf`] or 
[`BitcoinD::from_downloaded_with_conf`]

## Issues with traditional approach

I used integration testing based on external bash script launching needed external processes, there 
are many issues with this approach like:

* External script may interfere with local development environment [1](https://github.com/rust-bitcoin/rust-bitcoincore-rpc/blob/200fc8247c1896709a673b82a89ca0da5e7aa2ce/integration_test/run.sh#L9)
* Use of a single huge test to test everything [2](https://github.com/rust-bitcoin/rust-bitcoincore-rpc/blob/200fc8247c1896709a673b82a89ca0da5e7aa2ce/integration_test/src/main.rs#L122-L203)
* If test are separated, a failing test may fail to leave a clean situation, causing other test to 
fail (because of the initial situation, not a real failure)
* bash script are hard, especially support different OS and versions

## Features

  * It waits until bitcoind daemon become ready to accept RPC commands
  * `bitcoind` use a temporary directory as datadir. You can specify the root of your temp directories 
  so that you have node's datadir in a RAM disk (eg `/dev/shm`)
  * Free ports are asked to the OS. Since you can't reserve the given portm a low probability race 
  condition is still possible, for this reason the process is tried to be spawn 3 times with different
  ports.
  * The process is killed when the struct goes out of scope no matter how the test finishes
  * Allows easy spawning of dependent process like [electrs](https://github.com/RCasatta/electrsd)

Thanks to these features every `#[test]` could easily run isolated with its own environment.

## Doc

To build docs:

```sh
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --features download,doc --open
```

## MSRV

The MSRV is 1.48.0 for version 0.29.* if no feature is used, otherwise is 1.57

Note: to respect 1.48.0 MSRV you need to use and older version of the which and tempfile dependencies, 
like it's done in the CI:

```sh
cargo update -p serde --precise 1.0.152
cargo update -p log --precise 0.4.18
```

Pinning in `Cargo.toml` is avoided because it could cause
compilation issues downstream.

## Nix

For reproducibility reasons, Nix build scripts cannot hit the internet, but the
auto-download feature does exactly that. To successfully build under Nix the
user must provide the tarball locally and specify its location via the
`BITCOIND_TARBALL_FILE` env var.
Alternatively, use the dep without auto-download feature.

## Used by

* [firma](https://github.com/RCasatta/firma/)
* [payjoin](https://github.com/Kixunil/payjoin)
* [rust-miniscript](https://github.com/rust-bitcoin/rust-miniscript/tree/4a3ba11c2fd5063be960741d557f3f7a28041e1f/bitcoind-tests)

### Via bdk dependency

* [gun](https://github.com/LLFourn/gun)

### Via electrsd dependency:

* [bdk](https://github.com/bitcoindevkit/bdk)
* [BEWallet](https://github.com/LeoComandini/BEWallet)
* [gdk rust](https://github.com/Blockstream/gdk/blob/master/subprojects/gdk_rust/)
