//! Integration test: Create an entity and extend its BTL (blocks-to-live).

use alloy::{
    primitives::U256,
    signers::{local::LocalSigner, Signer},
};
use arkiv_sdk::{
    node_bindings::Arkiv,
    ops::{Create, Delete, Extend},
    rpc::types::{IncludeData, QueryOpts},
    tx::TransactionReceipt,
    Provider, ProviderBuilder, StorageProvider,
};
use std::time;

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::test]
#[serial_test::serial]
async fn test_extend_btl() -> Result<(), Box<dyn std::error::Error>> {
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

    // Create an entity with a short BTL
    let initial_btl = time::Duration::from_secs(20);
    let payload = b"Entity with extendable BTL";
    let pending_create = alice_provider
        .create_entities(vec![Create::new()
            .btl(initial_btl)
            .payload(payload.to_vec())
            .content_type("text/plain")
            .build()?])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let entity_key = created[0].entity_key;
    let initial_expiration = created[0].expiration_block;

    // Extend the BTL
    let extension_btl = time::Duration::from_secs(60);
    let pending_extend = alice_provider
        .extend_entities(vec![Extend::new(entity_key, extension_btl)])
        .await?;

    let receipt = pending_extend.get_receipt().await?;
    let TransactionReceipt { extended, .. } = receipt.try_into()?;
    assert_eq!(extended.len(), 1, "Expected one extended entity");
    assert_eq!(
        extended[0].entity_key, entity_key,
        "Extended entity key should match"
    );
    assert_eq!(
        extended[0].old_expiration_block, initial_expiration,
        "Old expiration should match initial"
    );
    assert!(
        extended[0].new_expiration_block > extended[0].old_expiration_block,
        "New expiration should be greater than old expiration"
    );

    // Clean up: delete the entity
    alice_provider
        .delete_entities(vec![Delete::new(entity_key)])
        .await?
        .get_receipt()
        .await?;

    Ok(())
}
