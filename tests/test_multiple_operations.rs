//! Integration test: Batch multiple operations in a single transaction.

use alloy::{
    primitives::U256,
    signers::{local::LocalSigner, Signer},
};
use arkiv_sdk::{
    node_bindings::Arkiv,
    ops::{Chown, Create, Delete, Update},
    rpc::types::{IncludeData, QueryOpts},
    tx::{StorageTransactionBuilder, TransactionReceipt},
    Provider, ProviderBuilder, StorageProvider,
};
use std::time;

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::test]
#[serial_test::serial]
async fn test_multiple_operations() -> Result<(), Box<dyn std::error::Error>> {
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

    // Create and fund Bob's account
    let bob = LocalSigner::random().with_chain_id(Some(arkiv.networkid()));
    let bob_address = bob.address();

    // First, create two entities separately
    let pending_create = alice_provider
        .create_entities(vec![
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Entity 1".to_vec())
                .content_type("text/plain")
                .build()?,
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Entity 2".to_vec())
                .content_type("text/plain")
                .build()?,
        ])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    assert_eq!(created.len(), 2, "Expected two created entities");

    let entity_key_1 = created[0].entity_key;
    let entity_key_2 = created[1].entity_key;

    // Now perform multiple operations in a single transaction:
    // - Update entity 1
    // - Transfer entity 2 to Bob
    let tx = alice_provider
        .storage_transaction()
        .update_entities(vec![Update::new()
            .entity_key(entity_key_1)
            .btl(time::Duration::from_secs(30))
            .payload(b"Updated Entity 1".to_vec())
            .content_type("text/plain")
            .build()?])
        .transfer_entities(vec![Chown::new(entity_key_2, bob_address)]);

    let pending_tx = alice_provider.send_storage_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;
    let TransactionReceipt {
        updated,
        transferred,
        ..
    } = receipt.try_into()?;

    // Verify both operations succeeded in the same transaction
    assert_eq!(updated.len(), 1, "Expected one updated entity");
    assert_eq!(updated[0].entity_key, entity_key_1);

    assert_eq!(transferred.len(), 1, "Expected one transferred entity");
    assert_eq!(transferred[0].entity_key, entity_key_2);
    assert_eq!(transferred[0].new_owner_address, bob_address);

    // Verify entity 1 was updated
    let query = alice_provider
        .query(
            &format!(r#"$key = {entity_key_1}"#),
            QueryOpts {
                include_data: Some(IncludeData {
                    payload: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let entity = &query["data"][0];
    let value = hex::decode(
        entity["value"]
            .as_str()
            .expect("Expected entity value of type string")
            .trim_start_matches("0x"),
    )?;
    assert_eq!(value.as_slice(), b"Updated Entity 1");

    // Clean up: delete both entities
    alice_provider
        .delete_entities(vec![Delete::new(entity_key_1)])
        .await?
        .get_receipt()
        .await?;

    Ok(())
}
