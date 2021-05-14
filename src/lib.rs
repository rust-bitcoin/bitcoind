#![warn(missing_docs)]

//!
//! Bitcoind
//!
//! Utility to run a regtest bitcoind process, useful in integration testing environment
//!
//! ```no_run
//! use bitcoincore_rpc::RpcApi;
//! let bitcoind = bitcoind::BitcoinD::new("/usr/local/bin/bitcoind").unwrap();
//! assert_eq!(0, bitcoind.client.get_blockchain_info().unwrap().blocks);
//! ```

use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::ffi::OsStr;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Struct representing the bitcoind process with related information
pub struct BitcoinD {
    /// Process child handle, used to terminate the process when this struct is dropped
    process: Child,
    /// Rpc client linked to this bitcoind process
    pub client: Client,
    /// Work directory, where the node store blocks and other stuff. It is kept in the struct so that
    /// directory is deleted only when this struct is dropped
    _work_dir: TempDir,
    /// Path to the node cookie file, useful for other client to connect to the node
    pub cookie_file: PathBuf,
    /// Url of the rpc of the node, useful for other client to connect to the node
    pub url: String,
}

/// All the possible error in this crate
#[derive(Debug)]
pub enum Error {
    /// No port available on the system
    PortUnavailable,
    /// Wrapper of io Error
    Io(std::io::Error),
    /// Wrapper of bitcoincore_rpc Error
    Rpc(bitcoincore_rpc::Error),
}

impl BitcoinD {
    /// Launch the bitcoind process from the given `exe` executable with default args
    /// Waits for the node to be ready before returning
    pub fn new<S: AsRef<OsStr>>(exe: S) -> Result<BitcoinD, Error> {
        BitcoinD::with_args(exe, vec![], false)
    }

    /// Launch the bitcoind process from the given `exe` executable with given `args`
    /// args must be a vector of String containing no spaces like `vec!["-dbcache=100".to_string()]`
    /// Waits for the node to be ready before returning
    pub fn with_args<S, I>(exe: S, args: I, view_stdout: bool) -> Result<BitcoinD, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let _work_dir = TempDir::new()?;
        let cookie_file = _work_dir.path().join("regtest").join(".cookie");
        let rpc_port = get_available_port().ok_or(Error::PortUnavailable)?;
        let url = format!("http://127.0.0.1:{}", rpc_port);
        let stdout = if view_stdout {
            Stdio::inherit()
        } else {
            Stdio::null()
        };

        let process = Command::new(exe)
            .arg(format!("-datadir={}", _work_dir.path().display()))
            .arg(format!("-rpcport={}", rpc_port))
            .arg("-regtest")
            .arg("-listen=0") // do not connect to p2p
            .arg("-fallbackfee=0.0001")
            .args(args)
            .stdout(stdout)
            .spawn()?;

        let node_url_default = format!("{}/wallet/default", url);
        // wait bitcoind is ready, use default wallet
        let client = loop {
            thread::sleep(Duration::from_millis(500));
            assert!(process.stderr.is_none());
            let client_result = Client::new(url.clone(), Auth::CookieFile(cookie_file.clone()));
            if let Ok(client_base) = client_result {
                if client_base.get_blockchain_info().is_ok() {
                    client_base
                        .create_wallet("default", None, None, None, None)
                        .unwrap();
                    break Client::new(node_url_default, Auth::CookieFile(cookie_file.clone()))
                        .unwrap();
                }
            }
        };

        Ok(BitcoinD {
            process,
            client,
            _work_dir,
            cookie_file,
            url,
        })
    }

    /// Stop the node, waiting correct process termination
    pub fn stop(&mut self) -> Result<ExitStatus, Error> {
        self.client.stop()?;
        Ok(self.process.wait()?)
    }
}

impl Drop for BitcoinD {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

/// Returns a non-used local port if available
/// Note there is a race condition during the time the method check availability and the caller
fn get_available_port() -> Option<u16> {
    // using 0 as port let the system assign a port available
    let t = TcpListener::bind(("127.0.0.1", 0)).ok()?;
    t.local_addr().ok().map(|s| s.port())
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<bitcoincore_rpc::Error> for Error {
    fn from(e: bitcoincore_rpc::Error) -> Self {
        Error::Rpc(e)
    }
}

#[cfg(test)]
mod test {
    use crate::BitcoinD;
    use bitcoincore_rpc::jsonrpc::serde_json::Value;
    use bitcoincore_rpc::RpcApi;
    use std::collections::HashMap;
    use std::env;

    #[test]
    fn test_bitcoind() {
        let exe = env::var("BITCOIND_EXE").expect("BITCOIND_EXE env var must be set");
        let bitcoind = BitcoinD::new(exe).unwrap();
        let info = bitcoind.client.get_blockchain_info().unwrap();
        assert_eq!(0, info.blocks);
        let address = bitcoind.client.get_new_address(None, None).unwrap();
        let _ = bitcoind.client.generate_to_address(1, &address).unwrap();
        let info = bitcoind.client.get_blockchain_info().unwrap();
        assert_eq!(1, info.blocks);
    }

    #[test]
    fn test_getindexinfo() {
        let exe = env::var("BITCOIND_EXE").expect("BITCOIND_EXE env var must be set");
        let bitcoind = BitcoinD::with_args(exe, vec!["-txindex".to_string()], false).unwrap();
        assert!(
            bitcoind.client.version().unwrap() >= 210_000,
            "getindexinfo requires bitcoin >0.21"
        );
        let info: HashMap<String, Value> = bitcoind.client.call("getindexinfo", &[]).unwrap();
        assert!(info.contains_key("txindex"));
        assert_eq!(bitcoind.client.version().unwrap(), 210_000);
    }
}
