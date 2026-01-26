//! Integration test: Create an entity and read it back to verify correctness.

use alloy::{
    hex::ToHexExt,
    primitives::U256,
    signers::{local::LocalSigner, Signer},
};
use arkiv_sdk::{
    node_bindings::Arkiv,
    ops::Create,
    rpc::types::{IncludeData, QueryOpts},
    tx::TransactionReceipt,
    Provider, ProviderBuilder, StorageProvider,
};
use std::time;

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::test]
#[serial_test::serial]
async fn test_create_and_read_entity() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn ephemeral Arkiv node
    let arkiv = Arkiv::default().spawn()?;

    // Create node provider for admin operations
    let node_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .connect_http(arkiv.endpoint_url())
        .erased();

    // Generate fee history
    arkiv_sdk::utils::generate_fee_history(&node_provider, arkiv.endpoint_url()).await;

    // Create and fund Alice's account
    let alice = LocalSigner::random().with_chain_id(Some(arkiv.networkid()));
    let alice_address = alice.address();
    let alice_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .fetch_chain_id()
        .wallet(alice)
        .connect_http(arkiv.endpoint_url())
        .erased();
    arkiv_sdk::utils::fund_account(&node_provider, alice_address, FAUCET_FUNDS).await;

    // Create an entity with test payload
    let payload = b"Hello, integration test!";
    let content_type = "text/plain";
    let btl = time::Duration::from_secs(30);

    let pending_create = alice_provider
        .create_entities(vec![Create::new()
            .btl(btl)
            .payload(payload.to_vec())
            .content_type(content_type)
            .build()?])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let entity_key = created[0].entity_key;

    // Query the entity back
    let query = alice_provider
        .query(
            &format!(r#"$key = {entity_key}"#),
            QueryOpts {
                include_data: Some(IncludeData {
                    key: true,
                    owner: true,
                    payload: true,
                    content_type: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    // Verify entity exists and data matches
    let entities = query["data"].as_array().unwrap();
    assert_eq!(entities.len(), 1, "Expected exactly one entity");

    let entity = &entities[0];

    // Verify owner
    assert_eq!(
        entity["owner"],
        alice_address.encode_hex_with_prefix(),
        "Owner address does not match"
    );

    // Verify content type
    assert_eq!(
        entity["contentType"].as_str().unwrap(),
        content_type,
        "Content type does not match"
    );

    // Verify payload
    let value = hex::decode(
        entity["value"]
            .as_str()
            .expect("Expected entity value of type string")
            .trim_start_matches("0x"),
    )?;
    assert_eq!(
        value.as_slice(),
        payload,
        "Payload value does not match"
    );

    Ok(())
}
