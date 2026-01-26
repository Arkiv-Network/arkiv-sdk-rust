//! Integration test: Create an entity, update it, and verify the update.

use alloy::{
    primitives::U256,
    signers::{local::LocalSigner, Signer},
};
use arkiv_sdk::{
    node_bindings::Arkiv,
    ops::{Create, Delete, Update},
    rpc::types::{IncludeData, QueryOpts},
    tx::TransactionReceipt,
    Provider, ProviderBuilder, StorageProvider,
};
use std::time;

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::test]
#[serial_test::serial]
async fn test_update_entity() -> Result<(), Box<dyn std::error::Error>> {
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

    // Create an entity with initial payload
    let initial_payload = b"Initial payload";
    let content_type = "text/plain";
    let btl = time::Duration::from_secs(30);

    let pending_create = alice_provider
        .create_entities(vec![Create::new()
            .btl(btl)
            .payload(initial_payload.to_vec())
            .content_type(content_type)
            .build()?])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let entity_key = created[0].entity_key;

    // Update the entity with new payload
    let updated_payload = b"Updated payload";
    let pending_update = alice_provider
        .update_entities(vec![Update::new()
            .entity_key(entity_key)
            .btl(btl)
            .payload(updated_payload.to_vec())
            .content_type(content_type)
            .build()?])
        .await?;

    let receipt = pending_update.get_receipt().await?;
    let TransactionReceipt { updated, .. } = receipt.try_into()?;
    assert_eq!(updated.len(), 1, "Expected one updated entity");

    // Query the entity to verify the update
    let query = alice_provider
        .query(
            &format!(r#"$key = {entity_key}"#),
            QueryOpts {
                include_data: Some(IncludeData {
                    payload: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let entities = query["data"].as_array().unwrap();
    assert_eq!(entities.len(), 1, "Expected exactly one entity");

    let entity = &entities[0];
    let value = hex::decode(
        entity["value"]
            .as_str()
            .expect("Expected entity value of type string")
            .trim_start_matches("0x"),
    )?;

    assert_eq!(
        value.as_slice(),
        updated_payload,
        "Payload should be updated"
    );
    assert_ne!(
        value.as_slice(),
        initial_payload,
        "Payload should not match initial value"
    );

    // Clean up: delete the entity
    alice_provider
        .delete_entities(vec![Delete::new(entity_key)])
        .await?
        .get_receipt()
        .await?;

    Ok(())
}
