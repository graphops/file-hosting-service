use ethers::signers::{
    coins_bip39::English, LocalWallet, MnemonicBuilder, Signer, Wallet, WalletError,
};
use ethers_core::k256::ecdsa::SigningKey;
use serde::Serialize;
use std::fs;
use std::{collections::HashMap, io};
use toml::Value;

/// Build Wallet from Private key or Mnemonic
pub fn build_wallet(value: &str) -> Result<Wallet<SigningKey>, WalletError> {
    value
        .parse::<LocalWallet>()
        .or(MnemonicBuilder::<English>::default().phrase(value).build())
}

/// Get wallet public address to String
pub fn wallet_address(wallet: &Wallet<SigningKey>) -> String {
    format!("{:?}", wallet.address())
}

/// Struct for version control
#[derive(Serialize, Debug, Clone)]
pub struct PackageVersion {
    pub version: String,
    pub dependencies: HashMap<String, String>,
}

/// Read the manfiest
fn read_manifest() -> Result<Value, anyhow::Error> {
    let toml_string = fs::read_to_string("subfile-exchange/Cargo.toml")
        .map_err(|e| anyhow::anyhow!("Could not read manifest: {e}"))?;
    let toml_value: Value = toml::from_str(&toml_string)
        .map_err(|e| anyhow::anyhow!("Could no read from manifest to toml: {e}"))?;
    Ok(toml_value)
}

/// Parse package versioning from the manifest
pub fn package_version() -> Result<PackageVersion, anyhow::Error> {
    read_manifest().map(|toml_file| {
        let pkg = toml_file.as_table().unwrap();
        let version = pkg
            .get("package")
            .and_then(|p| p.get("version"))
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let _dependencies = pkg.get("dependencies").and_then(|d| d.as_table()).unwrap();

        let release = PackageVersion {
            version,
            dependencies: HashMap::new(),
        };
        tracing::info!("Running package version {:#?}", release);

        release
    })
}

// Load public certificate from file.
#[allow(unused)]
fn load_certs(filename: &str) -> Result<Vec<rustls::Certificate>, anyhow::Error> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| anyhow::anyhow!(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|e| anyhow::anyhow!(format!("failed to load certificate: {:#?}", e)))?;
    Ok(certs.into_iter().map(rustls::Certificate).collect())
}

// Load private key from file.
#[allow(unused)]
fn load_private_key(filename: &str) -> Result<rustls::PrivateKey, anyhow::Error> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| anyhow::anyhow!(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .map_err(|e| anyhow::anyhow!("failed to load private key: {:#?}", e))?;
    if keys.len() != 1 {
        return Err(anyhow::anyhow!("expected a single private key"));
    }

    Ok(rustls::PrivateKey(keys[0].clone()))
}

/// Validate that private key as an Eth wallet
pub fn public_key(value: &str) -> Result<String, WalletError> {
    // The wallet can be stored instead of the original private key
    let wallet = build_wallet(value)?;
    let addr = wallet_address(&wallet);
    tracing::info!(address = addr, "Resolved Graphcast id");
    Ok(addr)
}
