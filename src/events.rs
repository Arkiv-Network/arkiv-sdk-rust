use alloy::eips::BlockNumberOrTag;
use alloy::providers::{DynProvider, Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::Log;
use alloy::rpc::types::eth::Filter;
use alloy::sol_types::{SolEvent, SolEventInterface};
use alloy::transports::http::reqwest::Url;
use anyhow::Result;
use futures::{Stream, StreamExt};
use std::convert::TryFrom;
use std::pin::Pin;

use crate::entity::Hash;
use crate::eth::{self, GolemBaseABI};

/// Represents a GolemBase event parsed from the blockchain log.
/// Used to distinguish between entity creation, update, and removal events.
#[derive(Debug)]
pub enum Event {
    /// Entity was created.
    /// Contains the entity ID, block number, and transaction hash.
    EntityCreated {
        /// The ID of the created entity
        entity_id: Hash,
        /// The expiration block of the entity
        expiration_block: u64,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: Hash,
    },
    /// Entity was updated.
    /// Contains the entity ID, block number, and transaction hash.
    EntityUpdated {
        /// The ID of the updated entity
        entity_id: Hash,
        /// The expiration block of the entity
        expiration_block: u64,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: Hash,
    },
    /// Entity was removed.
    /// Contains the entity ID, block number, and transaction hash.
    EntityRemoved {
        /// The ID of the removed entity
        entity_id: Hash,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: Hash,
    },
    /// Entity was extended.
    /// Contains the entity ID, block number, and transaction hash.
    EntityExtended {
        /// The ID of the removed entity
        entity_id: Hash,
        /// The old expiration block
        old_expiration_block: u64,
        /// The new expiration block
        new_expiration_block: u64,
        /// The block number where the event occurred
        block_number: u64,
        /// The transaction hash that triggered the event
        transaction_hash: Hash,
    },
}

impl TryFrom<Log> for Event {
    type Error = anyhow::Error;

    /// Attempts to parse a blockchain log into a `Event`.
    /// Returns an error if required fields are missing or the event type is unknown.
    fn try_from(log: Log) -> Result<Self> {
        let block_number = log
            .block_number
            .ok_or_else(|| anyhow::anyhow!("Missing block number"))?;
        let transaction_hash = log
            .transaction_hash
            .ok_or_else(|| anyhow::anyhow!("Missing transaction hash"))?;
        let parsed = GolemBaseABI::GolemBaseABIEvents::decode_log(&log.into())?;
        match parsed.data {
            GolemBaseABI::GolemBaseABIEvents::GolemBaseStorageEntityCreated(data) => {
                Ok(Event::EntityCreated {
                    entity_id: data.entityKey.into(),
                    expiration_block: data.expirationBlock.try_into().unwrap_or_default(),
                    block_number,
                    transaction_hash,
                })
            }
            GolemBaseABI::GolemBaseABIEvents::GolemBaseStorageEntityUpdated(data) => {
                Ok(Event::EntityUpdated {
                    entity_id: data.entityKey.into(),
                    expiration_block: data.expirationBlock.try_into().unwrap_or_default(),
                    block_number,
                    transaction_hash,
                })
            }
            GolemBaseABI::GolemBaseABIEvents::GolemBaseStorageEntityDeleted(data) => {
                Ok(Event::EntityRemoved {
                    entity_id: data.entityKey.into(),
                    block_number,
                    transaction_hash,
                })
            }
            GolemBaseABI::GolemBaseABIEvents::GolemBaseStorageEntityBTLExtended(data) => {
                Ok(Event::EntityExtended {
                    entity_id: data.entityKey.into(),
                    old_expiration_block: data.oldExpirationBlock.try_into().unwrap_or_default(),
                    new_expiration_block: data.newExpirationBlock.try_into().unwrap_or_default(),
                    block_number,
                    transaction_hash,
                })
            }
        }
    }
}

/// Client for subscribing to and streaming GolemBase events from the blockchain.
/// Provides methods to connect to a node and receive event streams for entity changes.
pub struct EventsClient {
    provider: DynProvider,
}

impl EventsClient {
    /// Creates a new `EventsClient` by connecting to the given websocket `Url`.
    /// Establishes a connection to the blockchain node for event streaming.
    pub async fn new(url: Url) -> anyhow::Result<Self> {
        log::debug!("Connecting to websocket provider: {url}");

        let provider = ProviderBuilder::new()
            .connect_ws(WsConnect::new(url.clone()))
            .await?
            .erased();

        log::info!("Connected to websocket provider: {url}");
        Ok(Self { provider })
    }

    /// Listens for GolemBase events from the blockchain, starting from the latest block.
    /// Returns a stream of parsed `Event` items that can be processed asynchronously.
    pub async fn events_stream<'a>(
        &'a self,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<Event>> + Send + 'a>>> {
        let filter = self.create_event_filter(BlockNumberOrTag::Latest);
        self.create_stream_from_filter(filter).await
    }

    /// Listens for GolemBase events starting from a specific block number.
    /// Returns a stream of parsed `Event` items from the given block onward.
    ///
    /// # Arguments
    /// * `block` - The block number to start listening for events from.
    pub async fn events_stream_from_block<'a>(
        &'a self,
        block: u64,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<Event>> + Send + 'a>>> {
        let filter = self.create_event_filter(BlockNumberOrTag::Number(block));
        self.create_stream_from_filter(filter).await
    }

    /// Creates a filter for GolemBase events, specifying the contract address and event signatures.
    fn create_event_filter(&self, block: BlockNumberOrTag) -> Filter {
        Filter::new()
            .address(eth::STORAGE_ADDRESS)
            .from_block(block)
            .events(vec![
                GolemBaseABI::GolemBaseStorageEntityCreated::SIGNATURE,
                GolemBaseABI::GolemBaseStorageEntityUpdated::SIGNATURE,
                GolemBaseABI::GolemBaseStorageEntityDeleted::SIGNATURE,
                GolemBaseABI::GolemBaseStorageEntityBTLExtended::SIGNATURE,
            ])
    }

    /// Creates a stream of events from a filter.
    /// Subscribes to logs matching the filter and maps them to `Event` values.
    async fn create_stream_from_filter<'a>(
        &'a self,
        filter: Filter,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<Event>> + Send + 'a>>> {
        let subscription = self.provider.subscribe_logs(&filter).await?;
        Ok(Box::pin(subscription.into_stream().map(Event::try_from)))
    }
}
