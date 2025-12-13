use std::ops::{Deref, DerefMut};

use alloy::{
    network::{Ethereum, Network, TransactionBuilder},
    primitives::{Address, TxKind},
};
use alloy_rlp::{Encodable, RlpDecodable, RlpEncodable};

pub mod ops;
pub mod receipt;

use ops::{chown::Chown, create::Create, delete::Delete, extend::Extend, update::Update};

use crate::network::StorageNetwork;

/// Extension trait for adding storage functionality to [`alloy::network::TransactionBuilder`].
pub trait StorageTransactionBuilder<S: StorageNetwork>:
    Default + Deref<Target = S::TransactionRequest> + DerefMut + Sized
{
    /// The address of the storage contract.
    const STORAGE_ADDRESS: Address;

    /// Access the underlying payload
    fn payload(&self) -> &S::Payload;
    fn payload_mut(&mut self) -> &mut S::Payload;

    /// Add a list of [`Create`] operations to the encodable transaction inputs.
    fn create_entities(mut self, creates: Vec<S::Create>) -> Self {
        self.payload_mut().with_creates(creates);
        self
    }

    /// Add a list of [`Update`] operations to the encodable transaction inputs.
    fn update_entities(mut self, updates: Vec<S::Update>) -> Self {
        self.payload_mut().with_updates(updates);
        self
    }

    /// Add a list of [`Delete`] operations to the encodable transaction inputs.
    fn delete_entities(mut self, deletes: Vec<S::Delete>) -> Self {
        self.payload_mut().with_deletes(deletes);
        self
    }

    /// Add a list of [`Extend`] operations to the encodable transaction inputs.
    fn extend_entities(mut self, extensions: Vec<S::Extend>) -> Self {
        self.payload_mut().with_extensions(extensions);
        self
    }

    /// Add a list of [`Chown`] operations to the encodable transaction inputs.
    fn transfer_entities(mut self, transfers: Vec<S::Chown>) -> Self {
        self.payload_mut().with_transfers(transfers);
        self
    }

    /// Encode the storage payload and set the transaction input and [`alloy::primitives::TxKind`],
    /// returning the inner [`alloy::network::Network::TransactionRequest`].
    fn into_request(mut self) -> S::TransactionRequest {
        let payload = self.payload();
        let mut input = Vec::with_capacity(payload.len());
        payload.encode(&mut input);

        self.set_kind(TxKind::Call(Self::STORAGE_ADDRESS));
        self.set_input(input);

        self.to_owned()
    }
}

/// The underlying network transaction to be sent and an encodable payload
/// containing storage operations. This type implements [`StorageTransactionBuilder`]
/// and is intended to be used just as [`alloy::network::TransactionRequest`].
pub struct StorageTransactionRequest<S: StorageNetwork> {
    payload: S::Payload,
    request: <S as Network>::TransactionRequest,
}
impl<S: StorageNetwork> Deref for StorageTransactionRequest<S> {
    type Target = <S as Network>::TransactionRequest;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}
impl<S: StorageNetwork> DerefMut for StorageTransactionRequest<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.request
    }
}
impl<S: StorageNetwork> Default for StorageTransactionRequest<S> {
    fn default() -> Self {
        let mut buf = Self {
            payload: Default::default(),
            request: Default::default(),
        };
        buf.set_kind(TxKind::Call(Self::STORAGE_ADDRESS));
        buf
    }
}
impl<S: StorageNetwork> StorageTransactionBuilder<S> for StorageTransactionRequest<S> {
    const STORAGE_ADDRESS: Address = crate::eth::STORAGE_ADDRESS;
    fn payload(&self) -> &S::Payload {
        &self.payload
    }
    fn payload_mut(&mut self) -> &mut S::Payload {
        &mut self.payload
    }
}

/// An RLP encodable storage transaction payload containing storage operations
/// to be executed on the network.
#[derive(Debug, Clone, Default, RlpEncodable, RlpDecodable)]
pub struct StoragePayload {
    /// A list of [`Create`] operations.
    pub creates: Vec<Create>,
    /// A list of [`Update`] operations.
    pub updates: Vec<Update>,
    /// A list of [`Delete`] operations.
    pub deletes: Vec<Delete>,
    /// A list of [`Extend`] operations.
    pub extensions: Vec<Extend>,
    /// A list of [`Chown`] operations.
    pub transfers: Vec<Chown>,
}

/// Update a mutable reference to a [`StorageNetwork`] payload.
///
/// This trait is necessary for mainitaining dynamic compatibility.
pub trait PayloadBuilder<S: StorageNetwork> {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn with_creates(&mut self, creates: Vec<S::Create>);
    fn with_updates(&mut self, updates: Vec<S::Update>);
    fn with_deletes(&mut self, deletes: Vec<S::Delete>);
    fn with_extensions(&mut self, extensions: Vec<S::Extend>);
    fn with_transfers(&mut self, transfers: Vec<S::Chown>);
}
impl PayloadBuilder<Ethereum> for StoragePayload {
    fn is_empty(&self) -> bool {
        self.creates.is_empty()
            && self.updates.is_empty()
            && self.deletes.is_empty()
            && self.extensions.is_empty()
            && self.transfers.is_empty()
    }
    fn len(&self) -> usize {
        self.creates.len()
            + self.updates.len()
            + self.deletes.len()
            + self.extensions.len()
            + self.transfers.len()
    }
    fn with_creates(&mut self, creates: Vec<<Ethereum as StorageNetwork>::Create>) {
        self.creates = creates;
    }
    fn with_updates(&mut self, updates: Vec<<Ethereum as StorageNetwork>::Update>) {
        self.updates = updates;
    }
    fn with_deletes(&mut self, deletes: Vec<<Ethereum as StorageNetwork>::Delete>) {
        self.deletes = deletes;
    }
    fn with_extensions(&mut self, extensions: Vec<<Ethereum as StorageNetwork>::Extend>) {
        self.extensions = extensions;
    }
    fn with_transfers(&mut self, transfers: Vec<<Ethereum as StorageNetwork>::Chown>) {
        self.transfers = transfers;
    }
}
