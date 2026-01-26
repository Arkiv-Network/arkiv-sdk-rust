//! Integration test: Create entities with string attributes and query by those attributes.

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
async fn test_string_attributes() -> Result<(), Box<dyn std::error::Error>> {
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

    // Create entities with string attributes
    let pending_create = alice_provider
        .create_entities(vec![
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Product 1".to_vec())
                .content_type("text/plain")
                .with_attribute(
                    Attribute::new("category", "electronics")
                        .map_into::<String>()
                )
                .with_attribute(
                    Attribute::new("status", "available")
                        .map_into::<String>()
                )
                .build()?,
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Product 2".to_vec())
                .content_type("text/plain")
                .with_attribute(
                    Attribute::new("category", "electronics")
                        .map_into::<String>()
                )
                .with_attribute(
                    Attribute::new("status", "sold")
                        .map_into::<String>()
                )
                .build()?,
            Create::new()
                .btl(time::Duration::from_secs(30))
                .payload(b"Product 3".to_vec())
                .content_type("text/plain")
                .with_attribute(
                    Attribute::new("category", "books")
                        .map_into::<String>()
                )
                .with_attribute(
                    Attribute::new("status", "available")
                        .map_into::<String>()
                )
                .build()?,
        ])
        .await?;

    let receipt = pending_create.get_receipt().await?;
    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let entity_keys: Vec<_> = created.iter().map(|e| e.entity_key).collect();

    // Query each entity back and verify the string attributes are stored correctly
    let expected_data = vec![
        ("Product 1", "electronics", "available"),
        ("Product 2", "electronics", "sold"),
        ("Product 3", "books", "available"),
    ];

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
        let attrs = entity["stringAttributes"].as_array().unwrap();

        // Verify category attribute
        let category_attr = attrs
            .iter()
            .find(|a| a["key"].as_str().unwrap() == "category")
            .expect("Category attribute should exist");
        let category = category_attr["value"].as_str().unwrap();

        // Verify status attribute
        let status_attr = attrs
            .iter()
            .find(|a| a["key"].as_str().unwrap() == "status")
            .expect("Status attribute should exist");
        let status = status_attr["value"].as_str().unwrap();

        // Verify the attribute values match what we created
        let (expected_product, expected_category, expected_status) = expected_data[i];
        assert_eq!(category, expected_category, "{} should have category {}", expected_product, expected_category);
        assert_eq!(status, expected_status, "{} should have status {}", expected_product, expected_status);
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
