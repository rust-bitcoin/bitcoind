#[cfg(feature = "26_0")]        // This is all features.
compile_error!{"bitcoind-json-rpc does not support bitcoind v26_0"}

#[cfg(all(feature = "25_1", not(feature = "26_0")))]
compile_error!{"bitcoind-json-rpc does not support bitcoind v25.1"}

#[cfg(all(feature = "25_0", not(feature = "25_1")))]
compile_error!{"bitcoind-json-rpc does not support bitcoind v25.0"}

#[cfg(all(feature = "24_0_1", not(feature = "25_0")))]
compile_error!{"bitcoind-json-rpc does not support bitcoind v24.0.1"}

#[cfg(all(feature = "23_1", not(feature = "24_0_1")))]
compile_error!{"bitcoind-json-rpc does not support bitcoind v23.1"}

#[cfg(all(feature = "22_1", not(feature = "23_1")))]
#[allow(unused_imports)]        // Not all users need the json types.
pub use bitcoind_json_rpc_client::{client_sync::v22::Client, json::v22 as json};

#[cfg(all(feature = "0_21_2", not(feature = "22_1")))]
compile_error!{"bitcoind-json-rpc does not support bitcoind v22.2"}

#[cfg(all(feature = "0_20_2", not(feature = "0_21_2")))]
compile_error!{"bitcoind-json-rpc does not support bitcoind v0.20.2"}

#[cfg(all(feature = "0_19_1", not(feature = "0_20_2")))]
compile_error!{"bitcoind-json-rpc does not support bitcoind v0.19.1"}

#[cfg(all(feature = "0_18_1", not(feature = "0_19_1")))]
#[allow(unused_imports)]        // Not all users need the json types.
pub use bitcoind_json_rpc_client::{client_sync::v18::Client, json::v18 as json};

#[cfg(all(feature = "0_17_1", not(feature = "0_18_1")))]
#[allow(unused_imports)]        // Not all users need the json types.
pub use bitcoind_json_rpc_client::{client_sync::v17::Client, json::v17 as json};
