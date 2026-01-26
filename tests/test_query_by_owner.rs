//! Integration test: Create multiple entities and query by owner address.

use alloy::{
    hex::ToHexExt,
    primitives::U256,
    signers::{local::LocalSigner, Signer},
};
use arkiv_sdk::{
    node_bindings::Arkiv,
    ops::{Create, Delete},
    rpc::types::{IncludeData, QueryOpts},
    tx::TransactionReceipt,
    Provider, ProviderBuilder, StorageProvider,
};
use std::time;

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::test]
#[serial_test::serial]
async fn test_query_by_owner() -> Result<(), Box<dyn std::error::Error>> {
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

    // Alice creates 3 entities
    let pending_create = alice_provider
        .create_entities(vec![
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Alice's entity 1".to_vec())
                .content_type("text/plain")
                .build()?,
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Alice's entity 2".to_vec())
                .content_type("text/plain")
                .build()?,
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Alice's entity 3".to_vec())
                .content_type("text/plain")
                .build()?,
        ])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let alice_keys: Vec<_> = created.iter().map(|e| e.entity_key).collect();

    // Bob creates 2 entities
    let pending_create = bob_provider
        .create_entities(vec![
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Bob's entity 1".to_vec())
                .content_type("text/plain")
                .build()?,
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Bob's entity 2".to_vec())
                .content_type("text/plain")
                .build()?,
        ])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let bob_keys: Vec<_> = created.iter().map(|e| e.entity_key).collect();

    // Query Alice's entities
    let query = alice_provider
        .query(
            &format!(r#"$owner = {alice_address}"#),
            QueryOpts {
                include_data: Some(IncludeData {
                    key: true,
                    owner: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let alice_entities = query["data"].as_array().unwrap();
    assert_eq!(
        alice_entities.len(),
        3,
        "Alice should have 3 entities"
    );

    // Verify all returned entities belong to Alice
    for entity in alice_entities {
        assert_eq!(
            entity["owner"],
            alice_address.encode_hex_with_prefix(),
            "All entities should belong to Alice"
        );
    }

    // Query Bob's entities
    let query = bob_provider
        .query(
            &format!(r#"$owner = {bob_address}"#),
            QueryOpts {
                include_data: Some(IncludeData {
                    key: true,
                    owner: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let bob_entities = query["data"].as_array().unwrap();
    assert_eq!(bob_entities.len(), 2, "Bob should have 2 entities");

    // Verify all returned entities belong to Bob
    for entity in bob_entities {
        assert_eq!(
            entity["owner"],
            bob_address.encode_hex_with_prefix(),
            "All entities should belong to Bob"
        );
    }

    // Clean up: delete all entities
    for key in alice_keys {
        alice_provider
            .delete_entities(vec![Delete::new(key)])
            .await?
            .get_receipt()
            .await?;
    }
    for key in bob_keys {
        bob_provider
            .delete_entities(vec![Delete::new(key)])
            .await?
            .get_receipt()
            .await?;
    }

    Ok(())
}
