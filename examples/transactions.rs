//! Spawn an ephemeral arkiv node, add faucet funds to an account and send storage transactions.

use std::io::{BufRead, Write};
use std::{fs, io, time};

use alloy::signers::local::LocalSigner;
use alloy::{
    hex::{FromHex, ToHexExt},
    primitives::{B256, U256},
    signers::Signer,
};
use arkiv_sdk::{
    Provider, ProviderBuilder, StorageProvider,
    node_bindings::Arkiv,
    ops::{Chown, Create, Delete, Update},
    rpc::types::{IncludeData, QueryOpts},
    tx::{StorageTransactionBuilder, TransactionReceipt},
};

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arkiv = Arkiv::default().keep_stderr().spawn()?;

    eprintln!(
        "transaction: arkiv node: pid: {}, networkid: {}, endpoint: {}\n",
        arkiv.id(),
        arkiv.networkid(),
        arkiv.endpoint_url()
    );

    let stderr = arkiv.stderr.take().expect("failed to get stderr handle");

    std::thread::spawn(|| {
        let mut reader = io::BufReader::new(stderr);
        let mut file = fs::File::create("transactions.log").unwrap();
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

    eprintln!("transaction: successfully generated fee history\n");

    let alice = LocalSigner::random().with_chain_id(Some(arkiv.networkid()));
    let alice_address = alice.address();
    let alice_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .fetch_chain_id()
        .wallet(alice)
        .connect_http(arkiv.endpoint_url())
        .erased();
    arkiv_sdk::utils::fund_account(&node_provider, alice_address, FAUCET_FUNDS).await;

    let bob = LocalSigner::random().with_chain_id(Some(arkiv.networkid()));
    let bob_address = bob.address();
    let bob_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .fetch_chain_id()
        .wallet(bob)
        .connect_http(arkiv.endpoint_url())
        .erased();
    arkiv_sdk::utils::fund_account(&node_provider, bob_address, FAUCET_FUNDS).await;

    let payload = "Hello, Bob!";
    let pending_create = alice_provider
        .create_entities(vec![
            Create::new()
                .btl(time::Duration::from_secs(10))
                .payload(payload)
                .content_type("plain/text")
                .build()?,
        ])
        .await?;
    let receipt = pending_create.get_receipt().await?;

    eprintln!("transaction: create_entities: {receipt:?}\n");

    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let entity_key = created[0].entity_key;

    let pending_transfer = alice_provider
        .transfer_entities(vec![Chown::new(entity_key, bob_address)])
        .await?;
    let receipt = pending_transfer.get_receipt().await?;

    eprintln!("transaction: transfer_entities: {receipt:?}\n");

    let query = alice_provider
        .query(
            &format!(r#"$key = {entity_key}"#),
            QueryOpts {
                include_data: Some(IncludeData {
                    owner: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let entity = &query["data"][0];

    assert_eq!(
        entity["owner"],
        bob_address.encode_hex_with_prefix(),
        "transaction: owner address does not match"
    );

    let query = bob_provider
        .query(
            &format!(r#"$owner = {bob_address} && $creator = {alice_address}"#),
            QueryOpts {
                include_data: Some(IncludeData {
                    key: true,
                    content_type: true,
                    payload: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let entity = &query["data"][0];
    let entity_key = B256::from_hex(entity["key"].as_str().unwrap().trim_start_matches("0x"))?;
    let content_type = entity["contentType"].as_str().unwrap();
    let value = hex::decode(
        entity["value"]
            .as_str()
            .expect("transaction: expected entity value of type string")
            .to_string()
            .trim_start_matches("0x"),
    )?;

    assert_eq!(
        value.as_slice(),
        payload.as_bytes(),
        "transaction: payload value does not match"
    );

    eprintln!(
        "transaction: Message from Alice: {}\n",
        String::from_utf8(value).unwrap()
    );

    let tx = bob_provider
        .storage_transaction()
        .update_entities(vec![
            Update::new()
                .entity_key(entity_key)
                .btl(time::Duration::from_secs(10))
                .payload("Hello, Alice!")
                .content_type(content_type)
                .build()?,
        ])
        .transfer_entities(vec![Chown::new(entity_key, alice_address)]);

    let pending_tx = bob_provider.send_storage_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;

    eprintln!("transaction: storage_transaction: {receipt:?}\n");

    let TransactionReceipt { transferred, .. } = receipt.try_into()?;

    let query = bob_provider
        .query(
            &format!(r#"$key = {}"#, transferred[0].entity_key),
            QueryOpts {
                include_data: Some(IncludeData {
                    owner: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let entity = &query["data"][0];

    assert_eq!(
        entity["owner"],
        alice_address.encode_hex_with_prefix(),
        "transaction: owner address does not match"
    );

    let query = alice_provider
        .query(
            &format!(r#"$owner = {alice_address} && $creator = {alice_address}"#),
            QueryOpts::default(),
        )
        .await?;

    let entity = &query["data"][0];
    let entity_key = B256::from_hex(entity["key"].as_str().unwrap().trim_start_matches("0x"))?;
    let value = hex::decode(
        entity["value"]
            .as_str()
            .expect("transaction: expected entity value of type string")
            .to_string()
            .trim_start_matches("0x"),
    )?;

    eprintln!(
        "transaction: Message from Bob: {}\n",
        String::from_utf8(value).unwrap()
    );

    let pending_delete = alice_provider
        .delete_entities(vec![Delete::new(entity_key)])
        .await?;

    let receipt = pending_delete.get_receipt().await?;

    eprintln!("transaction: delete_entities: {receipt:?}\n");

    let query = alice_provider
        .query(&format!(r#"$key = {entity_key}"#), QueryOpts::default())
        .await?;

    assert!(query["data"].as_array().unwrap().is_empty());

    Ok(())
}
