//! Integration test: Create entities with numeric attributes and query by those attributes.

use alloy::{
    primitives::U256,
    signers::{local::LocalSigner, Signer},
};
use arkiv_sdk::{
    entity::Attribute,
    node_bindings::Arkiv,
    ops::{Create, Delete},
    rpc::types::{IncludeData, QueryOpts},
    tx::{ops::WithAttribute, TransactionReceipt},
    Provider, ProviderBuilder, StorageProvider,
};
use std::time;

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);

#[tokio::test]
#[serial_test::serial]
async fn test_numeric_attributes() -> Result<(), Box<dyn std::error::Error>> {
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

    // Create entities with numeric attributes (level and score)
    let pending_create = alice_provider
        .create_entities(vec![
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Player A".to_vec())
                .content_type("text/plain")
                .with_attribute(
                    Attribute::new("level", 5u64)
                        .map_into::<u64>()
                )
                .with_attribute(
                    Attribute::new("score", 1000u64)
                        .map_into::<u64>()
                )
                .build()?,
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Player B".to_vec())
                .content_type("text/plain")
                .with_attribute(
                    Attribute::new("level", 10u64)
                        .map_into::<u64>()
                )
                .with_attribute(
                    Attribute::new("score", 2000u64)
                        .map_into::<u64>()
                )
                .build()?,
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Player C".to_vec())
                .content_type("text/plain")
                .with_attribute(
                    Attribute::new("level", 10u64)
                        .map_into::<u64>()
                )
                .with_attribute(
                    Attribute::new("score", 1500u64)
                        .map_into::<u64>()
                )
                .build()?,
        ])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let entity_keys: Vec<_> = created.iter().map(|e| e.entity_key).collect();

    // Query each entity back and verify the numeric attributes are stored correctly
    for (i, &entity_key) in entity_keys.iter().enumerate() {
        let query = alice_provider
            .query(
                &format!(r#"$key = {entity_key}"#),
                QueryOpts {
                    include_data: Some(IncludeData {
                        payload: true,
                        attributes: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;

        let entities = query["data"].as_array().unwrap();
        assert_eq!(entities.len(), 1, "Should find exactly one entity");

        let entity = &entities[0];
        let attrs = entity["numericAttributes"].as_array().unwrap();

        // Verify level attribute exists
        let level_attr = attrs
            .iter()
            .find(|a| a["key"].as_str().unwrap() == "level")
            .expect("Level attribute should exist");

        // Verify score attribute exists
        let score_attr = attrs
            .iter()
            .find(|a| a["key"].as_str().unwrap() == "score")
            .expect("Score attribute should exist");

        // Verify the attribute values match what we created
        let level = level_attr["value"].as_u64().unwrap();
        let score = score_attr["value"].as_u64().unwrap();

        match i {
            0 => {
                assert_eq!(level, 5, "Player A should have level 5");
                assert_eq!(score, 1000, "Player A should have score 1000");
            }
            1 => {
                assert_eq!(level, 10, "Player B should have level 10");
                assert_eq!(score, 2000, "Player B should have score 2000");
            }
            2 => {
                assert_eq!(level, 10, "Player C should have level 10");
                assert_eq!(score, 1500, "Player C should have score 1500");
            }
            _ => unreachable!(),
        }
    }

    // Clean up: delete all entities
    for key in entity_keys {
        alice_provider
            .delete_entities(vec![Delete::new(key)])
            .await?
            .get_receipt()
            .await?;
    }

    Ok(())
}
