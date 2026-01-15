//! Spawn an ephemeral arkiv node, add faucet funds to an account and send storage transactions.

use std::io::{BufRead, Write};
use std::{env, fs, io, path, time};

use alloy::{
    hex::ToHexExt,
    primitives::U256,
    signers::{Signer, local::LocalSigner},
};
use arkiv_sdk::{
    Provider, ProviderBuilder, StorageProvider,
    node_bindings::Arkiv,
    ops::Create,
    rpc::types::{IncludeData, QueryOpts},
    tx::{StorageTransactionBuilder, TransactionReceipt},
};

const KEYSTORE_PASSWORD: &str = "test";
const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arkiv = Arkiv::default().keep_stderr().ephemeral_datadir().spawn()?;

    eprintln!(
        "keystore-signer: arkiv node: pid: {}, networkid: {}, endpoint: {}\n",
        arkiv.id(),
        arkiv.networkid(),
        arkiv.endpoint_url()
    );

    let stderr = arkiv.stderr.take().expect("failed to get stderr handle");

    std::thread::spawn(|| {
        let mut reader = io::BufReader::new(stderr);
        let mut file = fs::File::create("keystore_signer.log").unwrap();
        let mut line = String::new();
        while reader.read_line(&mut line).unwrap() > 0 {
            file.write_all(line.as_bytes()).unwrap();
            line.clear();
        }
    });

    let node_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .connect_http(arkiv.endpoint_url())
        .erased();
    arkiv_sdk::utils::generate_fee_history(&node_provider, arkiv.endpoint_url()).await;

    eprintln!("keystore-signer: successfully generated fee history");

    let keystore_path = path::PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("keystore")
        .join("alice.json");
    let signer = LocalSigner::decrypt_keystore(keystore_path, KEYSTORE_PASSWORD)?
        .with_chain_id(Some(arkiv.networkid()));
    let address = signer.address();
    let wallet_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .fetch_chain_id()
        .wallet(signer)
        .connect_http(arkiv.endpoint_url())
        .erased();

    let receipt = arkiv_sdk::utils::fund_account(&node_provider, address, FAUCET_FUNDS).await;

    eprintln!("keystore-signer: fund_account: {receipt:?}\n");

    assert_eq!(
        FAUCET_FUNDS,
        wallet_provider.get_balance(address).await?,
        "keystore-signer: requested funds do not match current balance"
    );

    let payload = "Hello, world!";
    let tx = wallet_provider.storage_transaction().create_entities(vec![
        Create::new()
            .btl(time::Duration::from_secs(10))
            .payload(payload)
            .content_type("plain/text")
            .build()?,
    ]);

    let receipt = wallet_provider
        .send_storage_transaction(tx)
        .await?
        .get_receipt()
        .await?;

    eprintln!("keystore-signer: send_storage_transaction: {receipt:?}\n");

    let TransactionReceipt { created, .. } = receipt.try_into()?;

    let query = wallet_provider
        .query(
            &format!(r#"$key = {}"#, created[0].entity_key),
            QueryOpts {
                include_data: Some(IncludeData {
                    owner: true,
                    payload: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    eprintln!("keystore-signer: query: {query:?}");

    let entity = &query["data"][0];
    let value = hex::decode(
        entity["value"]
            .as_str()
            .expect("keystore-signer: expected entity value of type string")
            .to_string()
            .trim_start_matches("0x"),
    )?;

    assert_eq!(
        entity["owner"],
        address.encode_hex_with_prefix(),
        "keystore-signer: owner address does not match"
    );
    assert_eq!(
        value.as_slice(),
        payload.as_bytes(),
        "keystore-signer: payload value does not match"
    );

    Ok(())
}
