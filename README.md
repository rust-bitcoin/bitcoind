[![MIT license](https://img.shields.io/github/license/RCasatta/bitcoind)](https://github.com/RCasatta/bitcoind/blob/master/LICENSE)
[![Crates](https://img.shields.io/crates/v/bitcoind.svg)](https://crates.io/crates/bitcoind)

# Bitcoind

Utility to run a regtest bitcoind process, useful in integration testing environment.

```rust
use bitcoincore_rpc::RpcApi;
let exe_path = exe_path().expect("bitcoind executable must be provided in BITCOIND_EXE, or with a feature like '22_0', or be in PATH");
let bitcoind = bitcoind::BitcoinD::new(exe_path).unwrap();
assert_eq!(0, bitcoind.client.get_blockchain_info().unwrap().blocks);
```

## Automatic binaries download

When a feature like `22_0` is selected, the build script will automatically download the bitcoin core version `22.0`, verify the hashes and place it in the build directory for this crate.
Use utility function `downloaded_exe_path()` to get the downloaded executable path.

### Example 

In your project Cargo.toml, activate the following features

```toml

[dev-dependencies]
bitcoind = { version = "0.20.0", features = [ "22_0" ] }
```

Then use it:

```rust
let bitcoind = bitcoind::BitcoinD::new(bitcoind::downloaded_exe_path().unwrap()).unwrap();
```

## MSRV

### current 0.27+

1.53.0

### version 0.26.0

1.41.1

## Issues with traditional approach

I used integration testing based on external bash script launching needed external processes, there are many issues with this approach like:

* External script may interfere with local development environment https://github.com/rust-bitcoin/rust-bitcoincore-rpc/blob/200fc8247c1896709a673b82a89ca0da5e7aa2ce/integration_test/run.sh#L9
* Use of a single huge test to test everything https://github.com/rust-bitcoin/rust-bitcoincore-rpc/blob/200fc8247c1896709a673b82a89ca0da5e7aa2ce/integration_test/src/main.rs#L122-L203
* If test are separated, a failing test may fail to leave a clean situation, causing other test to fail (because of the initial situation, not a real failure)
* bash script are hard, especially support different OS and versions

## Features

  * It waits until bitcoind daemon become ready to accept RPC commands
  * bitcoind use a temporary directory as datadir. You can specify the root of your temp directories so that you have node's datadir in a RAM disk (eg `/dev/shm`)
  * Free ports are asked to the OS (a low probability race condition is still possible) 
  * the process is killed when the struct goes out of scope no matter how the test finishes
  * allows easy spawning of dependent process like https://github.com/RCasatta/electrsd

Thanks to these features every `#[test]` could easily run isolated with its own environment.

## Used by

* [firma](https://github.com/RCasatta/firma/)
* [payjoin](https://github.com/Kixunil/payjoin)

### Via bdk dependency

* [gun](https://github.com/LLFourn/gun)

### Via electrsd dependency:

* [bdk](https://github.com/bitcoindevkit/bdk)
* [BEWallet](https://github.com/LeoComandini/BEWallet)
* [gdk rust](https://github.com/Blockstream/gdk/blob/master/subprojects/gdk_rust/)
