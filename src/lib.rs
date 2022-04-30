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
use log::{debug, error, warn};
use std::ffi::OsStr;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Duration;
use std::{env, fmt, fs, thread};
use tempfile::TempDir;

pub extern crate bitcoincore_rpc;
pub extern crate tempfile;

/// Struct representing the bitcoind process with related information
pub struct BitcoinD {
    /// Process child handle, used to terminate the process when this struct is dropped
    process: Child,
    /// Rpc client linked to this bitcoind process
    pub client: Client,
    /// Work directory, where the node store blocks and other stuff.
    work_dir: DataDir,

    /// Contains information to connect to this node
    pub params: ConnectParams,
}

/// The DataDir struct defining the kind of data directory the node
/// will contain. Data directory can be either persistent, or temporary.
pub enum DataDir {
    /// Persistent Data Directory
    Persistent(PathBuf),
    /// Temporary Data Directory
    Temporary(TempDir),
}

impl DataDir {
    /// Return the data directory path
    fn path(&self) -> PathBuf {
        match self {
            Self::Persistent(path) => path.to_owned(),
            Self::Temporary(tmp_dir) => tmp_dir.path().to_path_buf(),
        }
    }
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
#[derive(Debug, PartialEq, Eq, Clone)]
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
pub enum Error {
    /// Wrapper of io Error
    Io(std::io::Error),
    /// Wrapper of bitcoincore_rpc Error
    Rpc(bitcoincore_rpc::Error),
    /// Returned when calling methods requiring a feature to be activated, but it's not
    NoFeature,
    /// Returned when calling methods requiring a env var to exist, but it's not
    NoEnvVar,
    /// Returned when calling methods requiring the bitcoind executable but none is found
    /// (no feature, no `BITCOIND_EXE`, no `bitcoind` in `PATH` )
    NoBitcoindExecutableFound,
    /// Returned when calling methods requiring either a feature or anv var, but both are present
    BothFeatureAndEnvVar,
    /// Wrapper of early exit status
    EarlyExit(ExitStatus),
    /// Returned when both tmpdir and staticdir is specified in `Conf` options
    BothDirsSpecified,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{:?}", e),
            Error::Rpc(e) => write!(f, "{:?}", e),
            Error::NoFeature => write!(f, "Called a method requiring a feature to be set, but it's not"),
            Error::NoEnvVar => write!(f, "Called a method requiring env var `BITCOIND_EXE` to be set, but it's not"),
            Error::NoBitcoindExecutableFound =>  write!(f, "`bitcoind` executable is required, provide it with one of the following: set env var `BITCOIND_EXE` or use a feature like \"22_0\" or have `bitcoind` executable in the `PATH`"),
            Error::BothFeatureAndEnvVar => write!(f, "Called a method requiring env var `BITCOIND_EXE` or a feature to be set, but both are set"),
            Error::EarlyExit(e) => write!(f, "The bitcoind process terminated early with exit code {}", e),
            Error::BothDirsSpecified => write!(f, "tempdir and staticdir cannot be enabled at same time in configuration options")
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

const LOCAL_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

/// The node configuration parameters, implements a convenient [Default] for most common use.
///
/// `#[non_exhaustive]` allows adding new parameters without breaking downstream users.
/// Users cannot instantiate the struct directly, they need to create it via the `default()` method
/// and mutate fields according to their preference.
///
/// Default values:
/// ```
/// let mut conf = bitcoind::Conf::default();
/// conf.args = vec!["-regtest", "-fallbackfee=0.0001"];
/// conf.view_stdout = false;
/// conf.p2p = bitcoind::P2P::No;
/// conf.network = "regtest";
/// conf.tmpdir = None;
/// conf.staticdir = None;
/// conf.attempts = 3;
/// assert_eq!(conf, bitcoind::Conf::default());
/// ```
///
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Conf<'a> {
    /// Bitcoind command line arguments containing no spaces like `vec!["-dbcache=300", "-regtest"]`
    /// note that `port`, `rpcport`, `connect`, `datadir`, `listen`
    /// cannot be used because they are automatically initialized.
    pub args: Vec<&'a str>,

    /// if `true` bitcoind log output will not be suppressed
    pub view_stdout: bool,

    /// Allows to specify options to open p2p port or connect to the another node
    pub p2p: P2P,

    /// Must match what specified in args without dashes, needed to locate the cookie file
    /// directory with different/esoteric networks
    pub network: &'a str,

    /// Optionally specify a temporary or persistent working directory for the node.
    /// The following two parameters can be configured to simulate desired working directory configuration.
    ///
    /// tmpdir is Some() && staticdir is Some() : Error. Cannot be enabled at same time.
    /// tmpdir is Some(temp_path) && staticdir is None : Create temporary directory at `tmpdir` path.
    /// tmpdir is None && staticdir is Some(work_path) : Create persistent directory at `staticdir` path.
    /// tmpdir is None && staticdir is None: Creates a temporary directory in OS default temporary directory (eg /tmp) or `TEMPDIR_ROOT` env variable path.
    ///
    /// It may be useful for example to set to a ramdisk via `TEMPDIR_ROOT` env option so that
    /// bitcoin nodes spawn very fast because their datadirs are in RAM. Should not be enabled with persistent
    /// mode, as it cause memory overflows.

    /// Temporary directory path
    pub tmpdir: Option<PathBuf>,

    /// Persistent directory path
    pub staticdir: Option<PathBuf>,

    /// Try to spawn the process `attempt` time
    ///
    /// The OS is giving available ports to use, however, they aren't booked, so it could rarely
    /// happen they are used at the time the process is spawn. When retrying other available ports
    /// are returned reducing the probability of conflicts to negligible.
    pub attempts: u8,
}

impl Default for Conf<'_> {
    fn default() -> Self {
        Conf {
            args: vec!["-regtest", "-fallbackfee=0.0001"],
            view_stdout: false,
            p2p: P2P::No,
            network: "regtest",
            tmpdir: None,
            staticdir: None,
            attempts: 3,
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
        let tmpdir = conf
            .tmpdir
            .clone()
            .or_else(|| env::var("TEMPDIR_ROOT").map(PathBuf::from).ok());
        let work_dir = match (&tmpdir, &conf.staticdir) {
            (Some(_), Some(_)) => return Err(Error::BothDirsSpecified),
            (Some(tmpdir), None) => DataDir::Temporary(TempDir::new_in(tmpdir)?),
            (None, Some(workdir)) => {
                fs::create_dir_all(workdir)?;
                DataDir::Persistent(workdir.to_owned())
            }
            (None, None) => DataDir::Temporary(TempDir::new()?),
        };

        let work_dir_path = work_dir.path();
        debug!("work_dir: {:?}", work_dir_path);
        let cookie_file = work_dir_path.join(conf.network).join(".cookie");
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

        let datadir_arg = format!("-datadir={}", work_dir_path.display());
        let rpc_arg = format!("-rpcport={}", rpc_port);
        let default_args = [&datadir_arg, &rpc_arg];

        debug!(
            "launching {:?} with args: {:?} {:?} AND custom args",
            exe.as_ref(),
            default_args,
            p2p_args
        );
        let mut process = Command::new(exe.as_ref())
            .args(&default_args)
            .args(&p2p_args)
            .args(&conf.args)
            .stdout(stdout)
            .spawn()?;

        let node_url_default = format!("{}/wallet/default", rpc_url);
        // wait bitcoind is ready, use default wallet
        let client = loop {
            if let Some(status) = process.try_wait()? {
                if conf.attempts > 0 {
                    warn!("early exit with: {:?}. Trying to launch again ({} attempts remaining), maybe some other process used our available port", status, conf.attempts);
                    let mut conf = conf.clone();
                    conf.attempts -= 1;
                    return Self::with_conf(exe, &conf);
                } else {
                    error!("early exit with: {:?}", status);
                    return Err(Error::EarlyExit(status));
                }
            }
            thread::sleep(Duration::from_millis(500));
            assert!(process.stderr.is_none());
            let client_result = Client::new(&rpc_url, Auth::CookieFile(cookie_file.clone()));
            if let Ok(client_base) = client_result {
                // RpcApi has get_blockchain_info method, however being generic with `Value` allows
                // to be compatible with different version, in the end we are only interested if
                // the call is succesfull not in the returned value.
                if client_base.call::<Value>("getblockchaininfo", &[]).is_ok() {
                    // Try creating new wallet, if fails due to already existing wallet file
                    // try loading the same. Return if still errors.
                    if client_base
                        .create_wallet("default", None, None, None, None)
                        .is_err()
                    {
                        client_base.load_wallet("default")?;
                    }
                    break Client::new(&node_url_default, Auth::CookieFile(cookie_file.clone()))?;
                }
            }
        };

        Ok(BitcoinD {
            process,
            client,
            work_dir,
            params: ConnectParams {
                datadir: work_dir_path,
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

    /// Return the current workdir path of the running node
    pub fn workdir(&self) -> PathBuf {
        self.work_dir.path()
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
            &self.rpc_url_with_wallet(wallet),
            Auth::CookieFile(self.params.cookie_file.clone()),
        )?)
    }
}

impl Drop for BitcoinD {
    fn drop(&mut self) {
        if let DataDir::Persistent(_) = self.work_dir {
            let _ = self.stop();
        }
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
pub fn downloaded_exe_path() -> Result<String, Error> {
    if versions::HAS_FEATURE {
        Ok(format!(
            "{}/bitcoin/bitcoin-{}/bin/bitcoind",
            env!("OUT_DIR"),
            versions::VERSION
        ))
    } else {
        Err(Error::NoFeature)
    }
}

/// Returns the daemon executable path if it's provided as a feature or as `BITCOIND_EXE` env var.
/// Returns error if none or both are set
pub fn exe_path() -> Result<String, Error> {
    match (downloaded_exe_path(), std::env::var("BITCOIND_EXE")) {
        (Ok(_), Ok(_)) => Err(Error::BothFeatureAndEnvVar),
        (Ok(path), Err(_)) => Ok(path),
        (Err(_), Ok(path)) => Ok(path),
        (Err(_), Err(_)) => which::which("bitcoind")
            .map_err(|_| Error::NoBitcoindExecutableFound)
            .map(|p| p.display().to_string()),
    }
}

#[cfg(test)]
mod test {
    use crate::bitcoincore_rpc::jsonrpc::serde_json::Value;
    use crate::bitcoincore_rpc::Client;
    use crate::exe_path;
    use crate::{get_available_port, BitcoinD, Conf, LOCAL_IP, P2P};
    use bitcoincore_rpc::RpcApi;
    use std::net::SocketAddrV4;
    use tempfile::TempDir;

    #[test]
    fn test_local_ip() {
        assert_eq!("127.0.0.1", format!("{}", LOCAL_IP));
        let port = get_available_port().unwrap();
        let socket = SocketAddrV4::new(LOCAL_IP, port);
        assert_eq!(format!("127.0.0.1:{}", port), format!("{}", socket));
    }

    #[test]
    fn test_bitcoind() {
        let exe = init();
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
        let mut conf = Conf::default();
        conf.p2p = P2P::Yes;

        let bitcoind = BitcoinD::with_conf(&exe, &conf).unwrap();
        assert_eq!(peers_connected(&bitcoind.client), 0);
        let mut other_conf = Conf::default();
        other_conf.p2p = bitcoind.p2p_connect(false).unwrap();

        let other_bitcoind = BitcoinD::with_conf(&exe, &other_conf).unwrap();
        assert_eq!(peers_connected(&bitcoind.client), 1);
        assert_eq!(peers_connected(&other_bitcoind.client), 1);
    }

    #[test]
    fn test_data_persistence() {
        // Create a Conf with staticdir type
        let mut conf = Conf::default();
        let datadir = TempDir::new().unwrap();
        conf.staticdir = Some(datadir.path().to_path_buf());

        // Start BitcoinD with persistent db config
        // Generate 101 blocks
        // Wallet balance should be 50
        let bitcoind = BitcoinD::with_conf(exe_path().unwrap(), &conf).unwrap();
        let core_addrs = bitcoind.client.get_new_address(None, None).unwrap();
        bitcoind
            .client
            .generate_to_address(101, &core_addrs)
            .unwrap();
        let wallet_balance_1 = bitcoind.client.get_balance(None, None).unwrap();
        let best_block_1 = bitcoind.client.get_best_block_hash().unwrap();

        drop(bitcoind);

        // Start a new BitcoinD with the same datadir
        let bitcoind = BitcoinD::with_conf(exe_path().unwrap(), &conf).unwrap();

        let wallet_balance_2 = bitcoind.client.get_balance(None, None).unwrap();
        let best_block_2 = bitcoind.client.get_best_block_hash().unwrap();

        // Check node chain data persists
        assert_eq!(best_block_1, best_block_2);

        // Check the node wallet balance persists
        assert_eq!(wallet_balance_1, wallet_balance_2);
    }

    #[test]
    fn test_multi_p2p() {
        let _ = env_logger::try_init();
        let mut conf_node1 = Conf::default();
        conf_node1.p2p = P2P::Yes;
        let node1 = BitcoinD::with_conf(exe_path().unwrap(), &conf_node1).unwrap();

        // Create Node 2 connected Node 1
        let mut conf_node2 = Conf::default();
        conf_node2.p2p = node1.p2p_connect(true).unwrap();
        let node2 = BitcoinD::with_conf(exe_path().unwrap(), &conf_node2).unwrap();

        // Create Node 3 Connected To Node
        let mut conf_node3 = Conf::default();
        conf_node3.p2p = node2.p2p_connect(false).unwrap();
        let node3 = BitcoinD::with_conf(exe_path().unwrap(), &conf_node3).unwrap();

        // Get each nodes Peers
        let node1_peers = peers_connected(&node1.client);
        let node2_peers = peers_connected(&node2.client);
        let node3_peers = peers_connected(&node3.client);

        // Peers found
        assert!(node1_peers >= 1);
        assert!(node2_peers >= 1);
        assert_eq!(node3_peers, 1, "listen false but more than 1 peer");
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

    fn peers_connected(client: &Client) -> usize {
        let result: Vec<Value> = client.call("getpeerinfo", &[]).unwrap();
        result.len()
    }

    fn init() -> String {
        let _ = env_logger::try_init();
        exe_path().unwrap()
    }
}
