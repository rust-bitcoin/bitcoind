# Bitcoind

Utility to run a regtest bitcoind process, useful in integration testing environment.

```
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
