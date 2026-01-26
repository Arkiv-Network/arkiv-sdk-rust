//! Integration test: Create an entity and transfer ownership to another address.

use alloy::{
    hex::ToHexExt,
    primitives::U256,
    signers::{local::LocalSigner, Signer},
};
use arkiv_sdk::{
    node_bindings::Arkiv,
    ops::{Chown, Create, Delete},
    rpc::types::{IncludeData, QueryOpts},
    tx::TransactionReceipt,
    Provider, ProviderBuilder, StorageProvider,
};
use std::time;

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::test]
#[serial_test::serial]
async fn test_transfer_ownership() -> Result<(), Box<dyn std::error::Error>> {
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
    let bob_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .fetch_chain_id()
        .wallet(bob)
        .connect_http(arkiv.endpoint_url())
        .erased();
    arkiv_sdk::utils::fund_account(&node_provider, bob_address, FAUCET_FUNDS).await;

    // Alice creates an entity
    let payload = b"Alice's entity";
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

    // Verify Alice is the owner
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
        alice_address.encode_hex_with_prefix(),
        "Initial owner should be Alice"
    );

    // Alice transfers ownership to Bob
    let pending_transfer = alice_provider
        .transfer_entities(vec![Chown::new(entity_key, bob_address)])
        .await?;

    let receipt = pending_transfer.get_receipt().await?;
    let TransactionReceipt { transferred, .. } = receipt.try_into()?;
    assert_eq!(transferred.len(), 1, "Expected one transferred entity");
    assert_eq!(
        transferred[0].entity_key, entity_key,
        "Transferred entity key should match"
    );
    assert_eq!(
        transferred[0].old_owner_address, alice_address,
        "Old owner should be Alice"
    );
    assert_eq!(
        transferred[0].new_owner_address, bob_address,
        "New owner should be Bob"
    );

    // Verify Bob is now the owner
    let query = bob_provider
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
        "Owner should now be Bob"
    );

    // Clean up: Bob deletes the entity
    bob_provider
        .delete_entities(vec![Delete::new(entity_key)])
        .await?
        .get_receipt()
        .await?;

    Ok(())
}
