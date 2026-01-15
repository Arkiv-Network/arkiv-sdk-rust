//! Spawn an ephemeral arkiv node, add faucet funds to an account and send storage transactions.

use std::time::Duration;

use alloy::{hex::ToHexExt, primitives::U256, signers::Signer};
use arkiv_sdk::{
    PrivateKeySigner, Provider, ProviderBuilder, StorageProvider,
    node_bindings::Arkiv,
    ops::Create,
    rpc::types::{IncludeData, QueryOpts},
    tx::{StorageTransactionBuilder, TransactionReceipt},
};

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

struct _Arkiv;
impl _Arkiv {
    fn id(&self) -> u32 {
        0
    }
    fn networkid(&self) -> u64 {
        1337
    }
    fn endpoint_url(&self) -> reqwest::Url {
        "http://0.0.0.0:8545".parse().unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arkiv = Arkiv::default().keep_stderr().ephemeral_datadir().spawn()?;

    eprintln!(
        "keystore-signer: arkiv node: pid: {}, networkid: {}, endpoint: {}\n",
        arkiv.id(),
        arkiv.networkid(),
        arkiv.endpoint_url()
    );

    let stderr = arkiv.stderr.take().unwrap();

    std::thread::spawn(|| {
        use std::io::{BufRead, Write};

        let mut reader = std::io::BufReader::new(stderr);
        let mut file = std::fs::File::create("arkiv.log").unwrap();

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

    let signer = PrivateKeySigner::random().with_chain_id(Some(arkiv.networkid()));
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
            .btl(Duration::from_secs(10))
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
    let entity_key = created[0].entity_key;

    let query = wallet_provider
        .query(
            &format!(r#"$key = {}"#, entity_key),
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
