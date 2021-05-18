# Bitcoind

Utility to run a regtest bitcoind process, useful in integration testing environment.

```
use bitcoincore_rpc::RpcApi;
let bitcoind = bitcoind::BitcoinD::new("/usr/local/bin/bitcoind").unwrap();
assert_eq!(0, bitcoind.client.get_blockchain_info().unwrap().blocks);
```

## Features

  * bitcoind use a temporary directory as datadir
  * A free port is asked to the OS (a low probability race condition is still possible) 
  * the process is killed when the struct goes out of scope no matter how the test finishes