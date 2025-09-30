use anyhow::Result;
use serial_test::serial;

use arkiv_sdk::entity::Create;
use arkiv_test_utils::get_client;

#[tokio::test]
#[serial]
async fn test_query_entities() -> Result<()> {
    let client = get_client()?;

    if let [entity1, entity2, entity3] = &client
        .create_entities(vec![
            Create::new(b"test1".to_vec(), 1000)
                .annotate_string("type", "test")
                .annotate_string("category", "alpha"),
            Create::new(b"test2".to_vec(), 1000)
                .annotate_string("type", "test")
                .annotate_string("category", "beta"),
            Create::new(b"test3".to_vec(), 1000)
                .annotate_string("type", "demo")
                .annotate_string("category", "alpha"),
        ])
        .await?[..]
    {
        // Test queries
        let type_test_entries = client.query_entity_keys("type = \"test\"").await?;
        tracing::info!("Entries with type = \"test\": {type_test_entries:?}");
        assert!(type_test_entries.contains(&entity1.entity_key));
        assert!(type_test_entries.contains(&entity2.entity_key));

        let category_alpha_entries = client.query_entity_keys("category = \"alpha\"").await?;
        tracing::info!("Entries with category = \"alpha\": {category_alpha_entries:?}");
        assert!(category_alpha_entries.contains(&entity1.entity_key));
        assert!(category_alpha_entries.contains(&entity3.entity_key));

        let type_demo_entries = client.query_entity_keys("type = \"demo\"").await?;
        tracing::info!("Entries with type = \"demo\": {type_demo_entries:?}");
        assert!(type_demo_entries.contains(&entity3.entity_key));

        let combined_and = client
            .query_entity_keys("type = \"test\" && category = \"beta\"")
            .await?;
        tracing::info!("Entries with type = \"test\" && category = \"beta\": {combined_and:?}");
        assert!(combined_and.contains(&entity2.entity_key));

        let combined_or = client
            .query_entity_keys("type = \"demo\" || category = \"beta\"")
            .await?;
        tracing::info!("Entries with type = \"demo\" || category = \"beta\": {combined_or:?}");
        assert!(combined_or.contains(&entity2.entity_key));
        assert!(combined_or.contains(&entity3.entity_key));

        // Test empty result
        let no_results = client.query_entity_keys("type = \"nonexistent\"").await?;
        tracing::info!("Entries with type = \"nonexistent\": {no_results:?}");
        assert_eq!(no_results.len(), 0);

        // Test selecting all entries
        let all_entries = client
            .query_entity_keys("type = \"test\" || type = \"demo\"")
            .await?;
        tracing::info!("All entries: {all_entries:?}");
        assert!(all_entries.contains(&entity1.entity_key));
        assert!(all_entries.contains(&entity2.entity_key));
        assert!(all_entries.contains(&entity3.entity_key));

        Ok(())
    } else {
        anyhow::bail!("Something went wrong with the transaction, this shouldn't happen!")
    }
}
