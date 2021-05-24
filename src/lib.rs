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

pub extern crate bitcoincore_rpc;
pub extern crate tempfile;

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
    pub rpc_url: String,
    /// p2p connection url, is some if the node started with p2p enabled
    pub p2p_url: Option<String>,
}

/// Enum to specify p2p settings
pub enum P2P {
    /// the node doesn't open a p2p port and work in standalone mode
    No,
    /// the node open a p2p port
    Yes,
    /// The node open a p2p port and also connects to the url given as parameter
    Connect(String),
}

/// All the possible error in this crate
#[derive(Debug)]
pub enum Error {
    /// Wrapper of io Error
    Io(std::io::Error),
    /// Wrapper of bitcoincore_rpc Error
    Rpc(bitcoincore_rpc::Error),
}

impl BitcoinD {
    /// Launch the bitcoind process from the given `exe` executable with default args
    /// Waits for the node to be ready before returning
    pub fn new<S: AsRef<OsStr>>(exe: S) -> Result<BitcoinD, Error> {
        BitcoinD::with_args(exe, vec![], false, P2P::No)
    }

    /// Launch the bitcoind process from the given `exe` executable with given `args`
    /// Waits for the node to be ready before returning
    /// `args` could be a vector of String containing no spaces like `vec!["-dbcache=100".to_string()]`
    /// `view_stdout` true will not suppress bitcoind log output
    /// `p2p` allows to specify options to open p2p port or connect to the another node
    /// `datadir` when None a temp directory is created as datadir, it will be deleted on drop
    ///  provide a directory when you don't want auto deletion (maybe because you can't control
    pub fn with_args<S, I>(exe: S, args: I, view_stdout: bool, p2p: P2P) -> Result<BitcoinD, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let _work_dir = TempDir::new()?;
        let datadir_path = _work_dir.path().to_path_buf();
        let cookie_file = datadir_path.join("regtest").join(".cookie");
        let rpc_port = get_available_port()?;
        let url = format!("http://127.0.0.1:{}", rpc_port);
        let (p2p_args, p2p_url) = match p2p {
            P2P::No => (vec!["-listen=0".to_string()], None),
            P2P::Yes => {
                let p2p_port = get_available_port()?;
                let p2p_url = format!("127.0.0.1:{}", p2p_port);
                let p2p_arg = format!("-port={}", p2p_port);
                let args = vec![p2p_arg];
                (args, Some(p2p_url))
            }
            P2P::Connect(other_node_url) => {
                let p2p_port = get_available_port()?;
                let p2p_url = format!("127.0.0.1:{}", p2p_port);
                let p2p_arg = format!("-port={}", p2p_port);
                let connect = format!("-connect={}", other_node_url);
                let args = vec![p2p_arg, connect];
                (args, Some(p2p_url))
            }
        };
        let stdout = if view_stdout {
            Stdio::inherit()
        } else {
            Stdio::null()
        };

        let process = Command::new(exe)
            .arg(format!("-datadir={}", datadir_path.display()))
            .arg(format!("-rpcport={}", rpc_port))
            .arg("-regtest")
            .arg("-fallbackfee=0.0001")
            .args(p2p_args)
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
            rpc_url: url,
            p2p_url,
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
pub fn get_available_port() -> Result<u16, Error> {
    // using 0 as port let the system assign a port available
    let t = TcpListener::bind(("127.0.0.1", 0))?; // 0 means the OS choose a free port
    Ok(t.local_addr().map(|s| s.port())?)
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
    use crate::{BitcoinD, P2P};
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
        let bitcoind =
            BitcoinD::with_args(exe, vec!["-txindex".to_string()], false, P2P::No).unwrap();
        assert!(
            bitcoind.client.version().unwrap() >= 210_000,
            "getindexinfo requires bitcoin >0.21"
        );
        let info: HashMap<String, Value> = bitcoind.client.call("getindexinfo", &[]).unwrap();
        assert!(info.contains_key("txindex"));
        assert_eq!(bitcoind.client.version().unwrap(), 210_000);
    }

    #[test]
    fn test_p2p() {
        let exe = env::var("BITCOIND_EXE").expect("BITCOIND_EXE env var must be set");
        let bitcoind = BitcoinD::with_args(exe.clone(), vec![], false, P2P::Yes).unwrap();
        assert_eq!(bitcoind.client.get_peer_info().unwrap().len(), 0);
        let other_bitcoind = BitcoinD::with_args(
            exe,
            vec![],
            false,
            P2P::Connect(bitcoind.p2p_url.clone().unwrap()),
        )
        .unwrap();
        assert_eq!(bitcoind.client.get_peer_info().unwrap().len(), 1);
        assert_eq!(other_bitcoind.client.get_peer_info().unwrap().len(), 1);
    }
}
