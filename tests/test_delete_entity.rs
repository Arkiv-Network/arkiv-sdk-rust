//! Integration test: Create an entity, delete it, and verify it's gone.

use alloy::{
    primitives::U256,
    signers::{local::LocalSigner, Signer},
};
use arkiv_sdk::{
    node_bindings::Arkiv,
    ops::{Create, Delete},
    rpc::types::QueryOpts,
    tx::TransactionReceipt,
    Provider, ProviderBuilder, StorageProvider,
};
use std::time;

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::test]
#[serial_test::serial]
async fn test_delete_entity() -> Result<(), Box<dyn std::error::Error>> {
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

    // Create an entity
    let payload = b"This will be deleted";
    let pending_create = alice_provider
        .create_entities(vec![Create::new()
            .btl(time::Duration::from_secs(30))
            .payload(payload.to_vec())
            .content_type("text/plain")
            .build()?])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let entity_key = created[0].entity_key;

    // Verify entity exists before deletion
    let query_before = alice_provider
        .query(&format!(r#"$key = {entity_key}"#), QueryOpts::default())
        .await?;
    assert_eq!(
        query_before["data"].as_array().unwrap().len(),
        1,
        "Entity should exist before deletion"
    );

    // Delete the entity
    let pending_delete = alice_provider
        .delete_entities(vec![Delete::new(entity_key)])
        .await?;

    let receipt = pending_delete.get_receipt().await?;
    let TransactionReceipt { deleted, .. } = receipt.try_into()?;
    assert_eq!(deleted.len(), 1, "Expected one deleted entity");
    assert_eq!(deleted[0].entity_key, entity_key, "Deleted entity key should match");

    // Verify entity no longer exists
    let query_after = alice_provider
        .query(&format!(r#"$key = {entity_key}"#), QueryOpts::default())
        .await?;
    assert!(
        query_after["data"].as_array().unwrap().is_empty(),
        "Entity should not exist after deletion"
    );

    Ok(())
}
