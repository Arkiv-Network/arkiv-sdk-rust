//! Module for Arkiv client functionality.
//! Exposes the main client interface for interacting with the Arkiv network.

use alloy::{
    network::Ethereum,
    primitives::U256,
    providers::{PendingTransactionBuilder, Provider},
    rpc::client::RpcCall,
    transports::TransportResult,
};

use crate::{
    entity::Entity,
    network::StorageNetwork,
    rpc::{ArkivRpcMethod, BlockTiming, QueryOpts},
    tx::StorageTransactionBuilder,
};

/// Adds storage methods to a [`alloy::providers::Provider`].
///
/// [`alloy::network` Documentation](https://github.com/alloy-rs/alloy/tree/f341fd655a2ae230adcefee3f39a55e7b87ab620/crates/network)
///
/// This trait maintains dynamic compatibility with [`alloy::providers::DynProvider`].
#[async_trait::async_trait]
pub trait StorageProvider<S: StorageNetwork>: Provider<S> + Send + Sync {
    /// Filter and retrieve entities matching a query expression.
    fn query(&self, query: &str, opts: QueryOpts) -> RpcCall<serde_json::Value, Vec<Entity>> {
        self.client()
            .request(ArkivRpcMethod::Query, serde_json::json!([query, opts]))
    }

    /// Returns total number of entities at current block.
    fn get_entity_count(&self) -> RpcCall<(), u64> {
        self.client().request(ArkivRpcMethod::GetEntityCount, ())
    }

    /// Returns storage slot usage accounting.
    fn get_number_of_used_slots(&self) -> RpcCall<(), U256> {
        self.client()
            .request(ArkivRpcMethod::GetNumberOfUsedSlots, ())
    }

    /// Returns current block timing information.
    fn get_block_timing(&self) -> RpcCall<(), BlockTiming> {
        self.client().request(ArkivRpcMethod::GetBlockTiming, ())
    }

    fn storage_transaction(&self) -> S::StorageTransactionRequest {
        Default::default()
    }

    async fn send_storage_transaction(
        &self,
        tx: S::StorageTransactionRequest,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_transaction(tx.into_request()).await
    }

    async fn create_entities(
        &self,
        creates: Vec<S::Create>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().create_entities(creates))
            .await
    }

    async fn update_entities(
        &self,
        updates: Vec<S::Update>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().update_entities(updates))
            .await
    }

    async fn delete_entities(
        &self,
        deletes: Vec<S::Delete>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().delete_entities(deletes))
            .await
    }

    async fn extend_entities(
        &self,
        extensions: Vec<S::Extend>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().extend_entities(extensions))
            .await
    }
}

/// Blanket implementation of the [`StorageProvider`] trait for [`alloy::network::Ethereum`] providers.
///
/// [`alloy::network` Documentation](https://github.com/alloy-rs/alloy/tree/f341fd655a2ae230adcefee3f39a55e7b87ab620/crates/network)
impl<T: Provider<Ethereum>> StorageProvider<Ethereum> for T {}

#[cfg(test)]
mod test {
    use alloy::{
        providers::{Provider, ProviderBuilder},
        signers::local::LocalSigner,
    };

    use crate::{
        EntityKey,
        entity::content_type::ContentType,
        providers::StorageProvider,
        tx::{
            StorageTransactionBuilder,
            ops::{create::Create, delete::Delete, update::Update},
        },
    };

    #[tokio::test]
    async fn test_arkiv_provider() {
        const PLAIN_TEXT: ContentType<&str> = ContentType::new("plain/text");

        let local_signer = LocalSigner::random();
        let client = ProviderBuilder::new()
            .wallet(local_signer)
            .connect_http("https://example.network".try_into().unwrap())
            .erased();

        let tx = client
            .storage_transaction()
            .create_entities(vec![
                Create::builder()
                    .btl(5000)
                    .content_type(PLAIN_TEXT)
                    .payload("test".as_bytes())
                    .build()
                    .unwrap(),
            ])
            .update_entities(vec![
                Update::builder()
                    .entity_key(EntityKey::default())
                    .btl(5000)
                    .content_type(PLAIN_TEXT)
                    .payload("test")
                    .build()
                    .unwrap(),
            ])
            .delete_entities(vec![Delete::new(EntityKey::default())]);

        assert!(client.send_storage_transaction(tx).await.is_ok());
    }
}
