//! Module for Arkiv client functionality.
//! Exposes the main client interface for interacting with the Arkiv network.

use alloy::{
    network::{Ethereum, Network},
    primitives::U256,
    providers::{DynProvider, PendingTransactionBuilder, Provider, ProviderBuilder},
    rpc::client::RpcCall,
    signers::local::LocalSigner,
    transports::TransportResult,
};

use crate::{
    EntityKey,
    entity::{Entity, content_type::ContentType},
    rpc::{ArkivRpcMethod, BlockTiming, QueryOpts},
    tx::{
        StorageTransactionBuilder, StorageTransactionRequest,
        ops::{create::Create, delete::Delete, extend::Extend, update::Update},
    },
};

/// Adds Arkiv methods to a [`alloy::network::Network`].
///
/// [`alloy::network` Documentation](https://github.com/alloy-rs/alloy/tree/f341fd655a2ae230adcefee3f39a55e7b87ab620/crates/network)
pub trait ArkivProviderExt<N: Network>: Provider<N> {
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

    fn create_entities<C, T>(
        &self,
        creates: C,
    ) -> impl Future<Output = TransportResult<PendingTransactionBuilder<N>>>
    where
        C: Into<Vec<Create>>,
        T: StorageTransactionBuilder<N> + From<Vec<Create>>,
    {
        self.send_storage_transaction(T::from(creates.into()))
    }

    fn update_entities<U, T>(
        &self,
        updates: U,
    ) -> impl Future<Output = TransportResult<PendingTransactionBuilder<N>>>
    where
        U: Into<Vec<Update>>,
        T: StorageTransactionBuilder<N> + From<Vec<Update>>,
    {
        self.send_storage_transaction(T::from(updates.into()))
    }

    fn delete_entities<D, T>(
        &self,
        deletes: D,
    ) -> impl Future<Output = TransportResult<PendingTransactionBuilder<N>>>
    where
        D: Into<Vec<Delete>>,
        T: StorageTransactionBuilder<N> + From<Vec<Delete>>,
    {
        self.send_storage_transaction(T::from(deletes.into()))
    }

    fn extend_entities<E, T>(
        &self,
        extensions: E,
    ) -> impl Future<Output = TransportResult<PendingTransactionBuilder<N>>>
    where
        E: Into<Vec<Extend>>,
        T: StorageTransactionBuilder<N> + From<Vec<Extend>>,
    {
        self.send_storage_transaction(T::from(extensions.into()))
    }

    fn send_storage_transaction<T: StorageTransactionBuilder<N>>(
        &self,
        tx: T,
    ) -> impl Future<Output = TransportResult<PendingTransactionBuilder<N>>> {
        self.send_transaction(tx.into_request())
    }
}

/// Blanket implementation of the [`Arkiv`] trait for [`alloy::network::Ethereum`] providers.
///
/// [`alloy::network` Documentation](https://github.com/alloy-rs/alloy/tree/f341fd655a2ae230adcefee3f39a55e7b87ab620/crates/network)
impl<T: Provider<Ethereum>> ArkivProviderExt<Ethereum> for T {}

#[tokio::test]
async fn test_arkiv_provider() {
    const PLAIN_TEXT: ContentType<&str> = "plain/text";

    let local_signer = LocalSigner::random();
    let client = ProviderBuilder::new()
        .wallet(local_signer)
        .connect_http("https://example.network")
        .erased();
    let tx = StorageTransactionRequest::default()
        .create_entities([Create::builder()
            .btl(5000)
            .content_type(PLAIN_TEXT)
            .payload(b"test")
            .build()])
        .update_entities([Update::builder()
            .entity_key(EntityKey::default())
            .btl(5000)
            .content_type(PLAIN_TEXT)
            .payload("test")
            .build()])
        .delete_entities(Delete::new(EntityKey::default()));

    assert!(client.send_storage_transaction(tx).await.is_ok());
}
