//! Module for Arkiv client functionality.
//! Exposes the main client interface for interacting with the Arkiv network.

#[cfg(feature = "pubsub")]
use alloy::rpc::types::{Filter, Log};
#[cfg(feature = "pubsub")]
use futures::StreamExt;

use alloy::{
    network::{Ethereum, TransactionBuilder},
    primitives::U256,
    providers::{PendingTransactionBuilder, Provider},
    rpc::client::RpcCall,
    transports::TransportResult,
};

use crate::{
    network::StorageNetwork,
    rpc::{
        ArkivRpcMethod,
        types::{BlockTiming, QueryOpts},
    },
    tx::{PayloadBuilder, StorageTransactionBuilder},
};

/// Adds storage methods to a [`alloy::providers::Provider`].
///
/// [`alloy::network` Documentation](https://github.com/alloy-rs/alloy/tree/f341fd655a2ae230adcefee3f39a55e7b87ab620/crates/network)
///
/// This trait maintains dynamic compatibility with [`alloy::providers::DynProvider`].
#[async_trait::async_trait]
pub trait StorageProvider<S: StorageNetwork>: Provider<S> + Send + Sync {
    /// Filter and retrieve entities matching a query expression.
    ///
    /// TODO:
    /// When we make a query, the returned value from the request is still JSON,
    /// however, this may be less idiomatic for practical use if the end user must
    /// do non-trivial work to get a decompressed, decoded and deserialized payload.
    fn query(
        &self,
        query_expression: &str,
        options: QueryOpts,
    ) -> RpcCall<serde_json::Value, serde_json::Value> {
        self.client().request(
            ArkivRpcMethod::Query,
            serde_json::json!([query_expression, options]),
        )
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

    /// Construct the default [`StorageNetwork::StorageTransactionRequest`].
    fn storage_transaction(&self) -> S::StorageTransactionRequest {
        Default::default()
    }

    async fn send_storage_transaction(
        &self,
        tx: S::StorageTransactionRequest,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_transaction(tx.into_request().with_chain_id(self.get_chain_id().await?))
            .await
    }

    async fn create_entities(
        &self,
        creates: Vec<<<S::StorageTransactionRequest as StorageTransactionBuilder<S>>::Payload as PayloadBuilder<S>>::Create>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().create_entities(creates))
            .await
    }

    async fn update_entities(
        &self,
        updates: Vec<<<S::StorageTransactionRequest as StorageTransactionBuilder<S>>::Payload as PayloadBuilder<S>>::Update>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().update_entities(updates))
            .await
    }

    async fn delete_entities(
        &self,
        deletes: Vec<<<S::StorageTransactionRequest as StorageTransactionBuilder<S>>::Payload as PayloadBuilder<S>>::Delete>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().delete_entities(deletes))
            .await
    }

    async fn extend_entities(
        &self,
        extensions: Vec<<<S::StorageTransactionRequest as StorageTransactionBuilder<S>>::Payload as PayloadBuilder<S>>::Extend>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().extend_entities(extensions))
            .await
    }

    async fn transfer_entities(
        &self,
        transfers: Vec<<<S::StorageTransactionRequest as StorageTransactionBuilder<S>>::Payload as PayloadBuilder<S>>::Chown>,
    ) -> TransportResult<PendingTransactionBuilder<S>> {
        self.send_storage_transaction(self.storage_transaction().transfer_entities(transfers))
            .await
    }

    /// A convenience method for subscribing to a stream of events from the [`StorageNetwork`]'s storage contract.
    /// Provides a closure over a [`alloy::rpc::types::Filter`] with the [`crate::tx::StorageTransactionRequest::STORAGE_ADDRESS`] pre-populated
    /// and attempts to convert the [`alloy::sol`] contract types into [`crate::network::StorageNetwork::StorageEvent`].
    #[cfg(feature = "pubsub")]
    async fn subscribe_storage_events(
        &self,
        f: fn(Filter) -> Filter,
    ) -> TransportResult<
        futures::stream::Map<
            alloy::pubsub::SubscriptionStream<Log>,
            fn(Log) -> Result<S::StorageEvent, <S::StorageEvent as TryFrom<Log>>::Error>,
        >,
    > {
        let subscription = self
            .subscribe_logs(&f(
                Filter::new().address(S::StorageTransactionRequest::STORAGE_ADDRESS)
            ))
            .await?;
        Ok(subscription.into_stream().map(S::StorageEvent::try_from))
    }
}

/// Blanket implementation of the [`StorageProvider`] trait for [`alloy::network::Ethereum`] providers.
///
/// [`alloy::network` Documentation](https://github.com/alloy-rs/alloy/tree/f341fd655a2ae230adcefee3f39a55e7b87ab620/crates/network)
impl<T: Provider<Ethereum>> StorageProvider<Ethereum> for T {}
