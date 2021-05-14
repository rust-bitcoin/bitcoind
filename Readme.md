# Bitcoind

Utility to run a regtest bitcoind process, useful in integration testing environment.

```
use bitcoincore_rpc::RpcApi;
let bitcoind = bitcoind::BitcoinD::new("/usr/local/bin/bitcoind").unwrap();
assert_eq!(0, bitcoind.client.get_blockchain_info().unwrap().blocks);
```
