use bitcoin_hashes::{sha256, Hash};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::str::FromStr;
use tar::Archive;

include!("src/versions.rs");

#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
fn download_filename() -> String {
    format!("bitcoin-{}-osx64.tar.gz", &VERSION)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn download_filename() -> String {
    format!("bitcoin-{}-x86_64-linux-gnu.tar.gz", &VERSION)
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
fn download_filename() -> String {
    format!("bitcoin-{}-aarch64-linux-gnu.tar.gz", &VERSION)
}

fn get_expected_sha256(filename: &str) -> Result<sha256::Hash, ()> {
    let sha256sums_filename = format!("sha256/bitcoin-core-{}-SHA256SUMS", &VERSION);
    #[cfg(any(
        feature = "0_21_1",
        feature = "0_21_0",
        feature = "0_20_1",
        feature = "0_20_0",
        feature = "0_19_1",
        feature = "0_19_0_1",
        feature = "0_18_1",
        feature = "0_18_0",
        feature = "0_17_1",
    ))]
    let sha256sums_filename = format!("{}.asc", sha256sums_filename);
    let file = File::open(sha256sums_filename).map_err(|_| ())?;
    for line in BufReader::new(file).lines().flatten() {
        let tokens: Vec<_> = line.split("  ").collect();
        if tokens.len() == 2 && filename == tokens[1] {
            return sha256::Hash::from_str(tokens[0]).map_err(|_| ());
        }
    }
    Err(())
}

fn main() {
    if !HAS_FEATURE {
        return;
    }
    let download_filename = download_filename();
    let expected_hash = get_expected_sha256(&download_filename).unwrap();
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let bitcoin_exe_home = Path::new(&out_dir).join("bitcoin");
    if !bitcoin_exe_home.exists() {
        std::fs::create_dir(&bitcoin_exe_home).unwrap();
    }
    let existing_filename = bitcoin_exe_home
        .join(format!("bitcoin-{}", VERSION))
        .join("bin")
        .join("bitcoind");

    if !existing_filename.exists() {
        println!(
            "filename:{} version:{} hash:{}",
            download_filename, VERSION, expected_hash
        );

        let url = format!(
            "https://bitcoincore.org/bin/bitcoin-core-{}/{}",
            VERSION, download_filename
        );
        let mut downloaded_bytes = Vec::new();

        let _size = ureq::get(&url)
            .call()
            .into_reader()
            .read_to_end(&mut downloaded_bytes)
            .unwrap();

        let downloaded_hash = sha256::Hash::hash(&downloaded_bytes);
        assert_eq!(expected_hash, downloaded_hash);
        let d = GzDecoder::new(&downloaded_bytes[..]);

        let mut archive = Archive::new(d);
        for mut entry in archive.entries().unwrap().flatten() {
            if let Ok(file) = entry.path() {
                if file.ends_with("bitcoind") {
                    entry.unpack_in(&bitcoin_exe_home).unwrap();
                }
            }
        }
    }
}
