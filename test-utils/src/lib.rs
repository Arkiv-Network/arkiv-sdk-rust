use alloy::providers::{Provider, ProviderBuilder};
use anyhow::Result;
use arkiv_sdk::{DynProvider, PrivateKeySigner};
use dirs::config_dir;
use url::Url;

/// Default URL for Arkiv node in tests
pub const ARKIV_URL: &str = "http://localhost:8545";
pub const ARKIV_WS_URL: &str = "ws://localhost:8545";

/// Default TTL value for test entities
pub const TEST_TTL: u64 = 30;

pub const TEST_KEYSTORE_PASSPHRASE: &str = "passphrase";

pub fn get_client() -> Result<DynProvider> {
    let keypath = config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join("arkiv")
        .join("wallet.json");
    let signer = PrivateKeySigner::decrypt_keystore(keypath, TEST_KEYSTORE_PASSPHRASE)?;
    let url = Url::parse(ARKIV_URL)?;
    let client = ProviderBuilder::new()
        .wallet(signer)
        .connect_http(url)
        .erased();
    Ok(client)
}
