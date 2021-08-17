[![MIT license](https://img.shields.io/github/license/RCasatta/bitcoind)](https://github.com/RCasatta/bitcoind/blob/master/LICENSE)
[![Crates](https://img.shields.io/crates/v/bitcoind.svg)](https://crates.io/crates/bitcoind)

# Bitcoind

Utility to run a regtest bitcoind process, useful in integration testing environment.

```rust
use bitcoincore_rpc::RpcApi;
let bitcoind = bitcoind::BitcoinD::new("/usr/local/bin/bitcoind").unwrap();
assert_eq!(0, bitcoind.client.get_blockchain_info().unwrap().blocks);
```

## Features

  * It waits until bitcoind daemon become ready to accept RPC commands
  * bitcoind use a temporary directory as datadir
  * Free ports are asked to the OS (a low probability race condition is still possible) 
  * the process is killed when the struct goes out of scope no matter how the test finishes
  * allows easy spawning of dependent process like https://github.com/RCasatta/electrsd

## Cargo features

When a feature like `0_21_1` is selected, the build script will automatically download the bitcoin core version 0.21.1
and verify the hashes and place it in the build directory for this crate.
Use utility function `downloaded_exe_path()` to get the downloaded executable path.

### Example

##### Cargo.toml

```toml

[dev-dependencies]
bitcoind = { version = "0.12.0", features = "0_21_1" }
```

#### In your tests

```rust
let bitcoind = bitcoind::BitcoinD::new(bitcoind::downloaded_exe_path().unwrap()).unwrap();
```

