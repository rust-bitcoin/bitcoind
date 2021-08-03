#![deny(missing_docs)]

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

mod versions;

use crate::bitcoincore_rpc::jsonrpc::serde_json::Value;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use log::debug;
use std::ffi::OsStr;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
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

    /// Contains information to connect to this node
    pub params: ConnectParams,
}

#[derive(Debug, Clone)]
/// Contains all the information to connect to this node
pub struct ConnectParams {
    /// Path to the node datadir
    pub datadir: PathBuf,
    /// Path to the node cookie file, useful for other client to connect to the node
    pub cookie_file: PathBuf,
    /// Url of the rpc of the node, useful for other client to connect to the node
    pub rpc_socket: SocketAddrV4,
    /// p2p connection url, is some if the node started with p2p enabled
    pub p2p_socket: Option<SocketAddrV4>,
}

/// Enum to specify p2p settings
pub enum P2P {
    /// the node doesn't open a p2p port and work in standalone mode
    No,
    /// the node open a p2p port
    Yes,
    /// The node open a p2p port and also connects to the url given as parameter, it's handy to
    /// initialize this with [BitcoinD::p2p_connect] of another node. The `bool` parameter indicates
    /// if the node can accept connection too.
    Connect(SocketAddrV4, bool),
}

/// All the possible error in this crate
#[derive(Debug)]
pub enum Error {
    /// Wrapper of io Error
    Io(std::io::Error),
    /// Wrapper of bitcoincore_rpc Error
    Rpc(bitcoincore_rpc::Error),
}

const LOCAL_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

/// The node configuration parameters, implements a convenient [Default] for most common use.
///
/// Default values:
/// ```no_run
/// bitcoind::Conf {
///     args: vec!["-regtest", "-fallbackfee=0.0001"],
///     view_stdout: false,
///     p2p: bitcoind::P2P::No,
///     network: "regtest",
/// };
/// ```
pub struct Conf<'a> {
    /// Bitcoind command line arguments containing no spaces like `vec!["-dbcache=300", "-regtest"]`
    /// note that `port`, `rpcport`, `connect`, `datadir`, `listen` cannot be used cause they are
    /// automatically initialized.
    pub args: Vec<&'a str>,

    /// if `true` bitcoind log output will not be suppressed
    pub view_stdout: bool,

    /// Allows to specify options to open p2p port or connect to the another node
    pub p2p: P2P,

    /// Must match what specified in args without dashes, needed to locate the cookie file
    /// directory with different/esoteric networks
    pub network: &'a str,
}

impl Default for Conf<'_> {
    fn default() -> Self {
        Conf {
            args: vec!["-regtest", "-fallbackfee=0.0001"],
            view_stdout: false,
            p2p: P2P::No,
            network: "regtest",
        }
    }
}

impl BitcoinD {
    /// Launch the bitcoind process from the given `exe` executable with default args.
    ///
    /// Waits for the node to be ready to accept connections before returning
    pub fn new<S: AsRef<OsStr>>(exe: S) -> Result<BitcoinD, Error> {
        BitcoinD::with_conf(exe, &Conf::default())
    }

    /// Launch the bitcoind process from the given `exe` executable with given [Conf] param
    pub fn with_conf<S: AsRef<OsStr>>(exe: S, conf: &Conf) -> Result<BitcoinD, Error> {
        let _work_dir = TempDir::new()?;
        let datadir = _work_dir.path().to_path_buf();
        let cookie_file = datadir.join(conf.network).join(".cookie");
        let rpc_port = get_available_port()?;
        let rpc_socket = SocketAddrV4::new(LOCAL_IP, rpc_port);
        let rpc_url = format!("http://{}", rpc_socket);
        let (p2p_args, p2p_socket) = match conf.p2p {
            P2P::No => (vec!["-listen=0".to_string()], None),
            P2P::Yes => {
                let p2p_port = get_available_port()?;
                let p2p_socket = SocketAddrV4::new(LOCAL_IP, p2p_port);
                let p2p_arg = format!("-port={}", p2p_port);
                let args = vec![p2p_arg];
                (args, Some(p2p_socket))
            }
            P2P::Connect(other_node_url, listen) => {
                let p2p_port = get_available_port()?;
                let p2p_socket = SocketAddrV4::new(LOCAL_IP, p2p_port);
                let p2p_arg = format!("-port={}", p2p_port);
                let connect = format!("-connect={}", other_node_url);
                let mut args = vec![p2p_arg, connect];
                if listen {
                    args.push("-listen=1".to_string())
                }
                (args, Some(p2p_socket))
            }
        };
        let stdout = if conf.view_stdout {
            Stdio::inherit()
        } else {
            Stdio::null()
        };

        let datadir_arg = format!("-datadir={}", datadir.display());
        let rpc_arg = format!("-rpcport={}", rpc_port);
        let default_args = [&datadir_arg, &rpc_arg];

        debug!(
            "launching {:?} with args: {:?} {:?} AND custom args",
            exe.as_ref(),
            default_args,
            p2p_args
        );
        let process = Command::new(exe)
            .args(&default_args)
            .args(&p2p_args)
            .args(&conf.args)
            .stdout(stdout)
            .spawn()?;

        let node_url_default = format!("{}/wallet/default", rpc_url);
        // wait bitcoind is ready, use default wallet
        let client = loop {
            thread::sleep(Duration::from_millis(500));
            assert!(process.stderr.is_none());
            let client_result = Client::new(rpc_url.clone(), Auth::CookieFile(cookie_file.clone()));
            if let Ok(client_base) = client_result {
                // RpcApi has get_blockchain_info method, however being generic with `Value` allows
                // to be compatible with different version, in the end we are only interested if
                // the call is succesfull not in the returned value.
                if client_base.call::<Value>("getblockchaininfo", &[]).is_ok() {
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
            params: ConnectParams {
                datadir,
                cookie_file,
                rpc_socket,
                p2p_socket,
            },
        })
    }

    /// Returns the rpc URL including the schema eg. http://127.0.0.1:44842
    pub fn rpc_url(&self) -> String {
        format!("http://{}", self.params.rpc_socket)
    }

    #[cfg(not(any(feature = "0_17_1", feature = "0_18_0", feature = "0_18_1")))]
    /// Returns the rpc URL including the schema and the given `wallet_name`
    /// eg. http://127.0.0.1:44842/wallet/my_wallet
    pub fn rpc_url_with_wallet<T: AsRef<str>>(&self, wallet_name: T) -> String {
        format!(
            "http://{}/wallet/{}",
            self.params.rpc_socket,
            wallet_name.as_ref()
        )
    }

    /// Returns the [P2P] enum to connect to this node p2p port
    pub fn p2p_connect(&self, listen: bool) -> Option<P2P> {
        self.params.p2p_socket.map(|s| P2P::Connect(s, listen))
    }

    /// Stop the node, waiting correct process termination
    pub fn stop(&mut self) -> Result<ExitStatus, Error> {
        self.client.stop()?;
        Ok(self.process.wait()?)
    }

    #[cfg(not(any(feature = "0_17_1", feature = "0_18_0", feature = "0_18_1")))]
    /// Create a new wallet in the running node, and return an RPC client connected to the just
    /// created wallet
    pub fn create_wallet<T: AsRef<str>>(&self, wallet: T) -> Result<Client, Error> {
        let _ = self
            .client
            .create_wallet(wallet.as_ref(), None, None, None, None)?;
        Ok(Client::new(
            self.rpc_url_with_wallet(wallet),
            Auth::CookieFile(self.params.cookie_file.clone()),
        )?)
    }
}

impl Drop for BitcoinD {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

/// Returns a non-used local port if available.
///
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

/// Provide the bitcoind executable path if a version feature has been specified
pub fn downloaded_exe_path() -> Option<String> {
    // CARGO_HOME surely available only in `build.rs` here we need to get from home_dir
    if versions::HAS_FEATURE {
        Some(format!(
            "{}/bitcoin/bitcoin-{}/bin/bitcoind",
            home::cargo_home().ok()?.display(),
            versions::VERSION
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use crate::downloaded_exe_path;
    use crate::{get_available_port, BitcoinD, Conf, LOCAL_IP, P2P};
    use bitcoincore_rpc::RpcApi;
    use std::env;
    use std::net::SocketAddrV4;

    #[test]
    fn test_local_ip() {
        assert_eq!("127.0.0.1", format!("{}", LOCAL_IP));
        let port = get_available_port().unwrap();
        let socket = SocketAddrV4::new(LOCAL_IP, port);
        assert_eq!(format!("127.0.0.1:{}", port), format!("{}", socket));
    }

    #[test]
    fn test_default() {
        let conf = Conf {
            p2p: P2P::Yes,
            ..Default::default()
        };
        assert_eq!("regtest", conf.network);
    }

    #[test]
    fn test_bitcoind() {
        let exe = init();
        println!("{}", exe);
        let bitcoind = BitcoinD::new(exe).unwrap();
        let info = bitcoind.client.get_blockchain_info().unwrap();
        assert_eq!(0, info.blocks);
        let address = bitcoind.client.get_new_address(None, None).unwrap();
        let _ = bitcoind.client.generate_to_address(1, &address).unwrap();
        let info = bitcoind.client.get_blockchain_info().unwrap();
        assert_eq!(1, info.blocks);
    }

    #[test]
    #[cfg(any(feature = "0_21_0", feature = "0_21_1"))]
    fn test_getindexinfo() {
        let exe = init();
        let mut conf = Conf::default();
        conf.args.push("-txindex");
        let bitcoind = BitcoinD::with_conf(&exe, &conf).unwrap();
        assert!(
            bitcoind.client.version().unwrap() >= 210_000,
            "getindexinfo requires bitcoin >0.21"
        );
        let info: std::collections::HashMap<String, bitcoincore_rpc::jsonrpc::serde_json::Value> =
            bitcoind.client.call("getindexinfo", &[]).unwrap();
        assert!(info.contains_key("txindex"));
        assert!(bitcoind.client.version().unwrap() >= 210_000);
    }

    #[test]
    fn test_p2p() {
        let exe = init();
        let conf = Conf {
            p2p: P2P::Yes,
            ..Default::default()
        };
        let bitcoind = BitcoinD::with_conf(&exe, &conf).unwrap();
        assert_eq!(bitcoind.client.get_peer_info().unwrap().len(), 0);
        let other_conf = Conf {
            p2p: bitcoind.p2p_connect(false).unwrap(),
            ..Default::default()
        };
        let other_bitcoind = BitcoinD::with_conf(&exe, &other_conf).unwrap();
        assert_eq!(bitcoind.client.get_peer_info().unwrap().len(), 1);
        assert_eq!(other_bitcoind.client.get_peer_info().unwrap().len(), 1);
    }

    #[test]
    fn test_multi_p2p() {
        let conf_node1 = Conf {
            p2p: P2P::Yes,
            ..Default::default()
        };
        let node1 = BitcoinD::with_conf(exe_path(), &conf_node1).unwrap();

        // Create Node 2 connected Node 1
        let conf_node2 = Conf {
            p2p: node1.p2p_connect(true).unwrap(),
            ..Default::default()
        };
        let node2 = BitcoinD::with_conf(exe_path(), &conf_node2).unwrap();

        // Create Node 3 Connected To Node 2
        let conf_node3 = Conf {
            p2p: node2.p2p_connect(false).unwrap(),
            ..Default::default()
        };
        let node3 = BitcoinD::with_conf(exe_path(), &conf_node3).unwrap();

        // Get each nodes Peers
        let node1_peers = node1.client.get_peer_info().unwrap();
        let node2_peers = node2.client.get_peer_info().unwrap();
        let node3_peers = node3.client.get_peer_info().unwrap();

        // Peers found
        assert!(node1_peers.len() >= 1);
        assert!(node2_peers.len() >= 1);
        assert_eq!(node3_peers.len(), 1, "listen false but more than 1 peer");
    }

    #[cfg(not(any(feature = "0_17_1", feature = "0_18_0", feature = "0_18_1")))]
    #[test]
    fn test_multi_wallet() {
        use bitcoincore_rpc::bitcoin::Amount;
        let exe = init();
        let bitcoind = BitcoinD::new(exe).unwrap();
        let alice = bitcoind.create_wallet("alice").unwrap();
        let alice_address = alice.get_new_address(None, None).unwrap();
        let bob = bitcoind.create_wallet("bob").unwrap();
        let bob_address = bob.get_new_address(None, None).unwrap();
        bitcoind
            .client
            .generate_to_address(1, &alice_address)
            .unwrap();
        bitcoind
            .client
            .generate_to_address(101, &bob_address)
            .unwrap();
        assert_eq!(
            Amount::from_btc(50.0).unwrap(),
            alice.get_balances().unwrap().mine.trusted
        );
        assert_eq!(
            Amount::from_btc(50.0).unwrap(),
            bob.get_balances().unwrap().mine.trusted
        );
        assert_eq!(
            Amount::from_btc(5000.0).unwrap(),
            bob.get_balances().unwrap().mine.immature
        );
        alice
            .send_to_address(
                &bob_address,
                Amount::from_btc(1.0).unwrap(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        assert!(
            alice.get_balances().unwrap().mine.trusted < Amount::from_btc(49.0).unwrap()
                && alice.get_balances().unwrap().mine.trusted > Amount::from_btc(48.9).unwrap()
        );
        assert_eq!(
            Amount::from_btc(1.0).unwrap(),
            bob.get_balances().unwrap().mine.untrusted_pending
        );
        assert!(
            bitcoind.create_wallet("bob").is_err(),
            "wallet already exist"
        );
    }

    fn exe_path() -> String {
        if let Some(downloaded_exe_path) = downloaded_exe_path() {
            downloaded_exe_path
        } else {
            env::var("BITCOIND_EXE").expect(
                "when no version feature is specified, you must specify BITCOIND_EXE env var",
            )
        }
    }

    fn init() -> String {
        let _ = env_logger::try_init();
        exe_path()
    }
}
