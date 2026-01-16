use std::{
    io::Write,
    ops::{Deref, DerefMut},
};

use ::brotli::CompressorWriter;
use alloy::{
    network::{Ethereum, Network, TransactionBuilder},
    primitives::{Address, TxKind},
};
use alloy_rlp::{Encodable, RlpDecodable, RlpEncodable};

use crate::network::StorageNetwork;

pub mod ops;
pub mod receipt;

use ops::{Chown, Create, Delete, Extend, Update};
pub use receipt::{
    ChownReceipt, CreateReceipt, DeleteReceipt, ExtendReceipt, TransactionReceipt, UpdateReceipt,
};

/// Extension trait for adding storage functionality to [`alloy::network::TransactionBuilder`].
pub trait StorageTransactionBuilder<S: StorageNetwork>:
    Default + Deref<Target = S::TransactionRequest> + DerefMut + Sized
{
    /// The address of the storage contract.
    const STORAGE_ADDRESS: Address;

    type Payload: Default + alloy_rlp::Encodable + PayloadBuilder<S> + Send;

    /// Access the underlying payload
    fn payload(&self) -> &Self::Payload;
    fn payload_mut(&mut self) -> &mut Self::Payload;

    /// Add a list of [`Create`] operations to the encodable transaction inputs.
    fn create_entities(
        mut self,
        creates: Vec<<Self::Payload as PayloadBuilder<S>>::Create>,
    ) -> Self {
        self.payload_mut().with_creates(creates);
        self
    }

    /// Add a list of [`Update`] operations to the encodable transaction inputs.
    fn update_entities(
        mut self,
        updates: Vec<<Self::Payload as PayloadBuilder<S>>::Update>,
    ) -> Self {
        self.payload_mut().with_updates(updates);
        self
    }

    /// Add a list of [`Delete`] operations to the encodable transaction inputs.
    fn delete_entities(
        mut self,
        deletes: Vec<<Self::Payload as PayloadBuilder<S>>::Delete>,
    ) -> Self {
        self.payload_mut().with_deletes(deletes);
        self
    }

    /// Add a list of [`Extend`] operations to the encodable transaction inputs.
    fn extend_entities(
        mut self,
        extensions: Vec<<Self::Payload as PayloadBuilder<S>>::Extend>,
    ) -> Self {
        self.payload_mut().with_extensions(extensions);
        self
    }

    /// Add a list of [`Chown`] operations to the encodable transaction inputs.
    fn transfer_entities(
        mut self,
        transfers: Vec<<Self::Payload as PayloadBuilder<S>>::Chown>,
    ) -> Self {
        self.payload_mut().with_transfers(transfers);
        self
    }

    /// Encode and compress the storage payload and set the transaction input and [`alloy::primitives::TxKind`],
    /// returning the inner [`alloy::network::Network::TransactionRequest`].
    fn into_request(self) -> S::TransactionRequest {
        let payload = self.payload();
        let mut encoded = Vec::with_capacity(payload.len());
        payload.encode(&mut encoded);

        let mut compressed = Vec::with_capacity(encoded.len());
        let mut writer = CompressorWriter::new(&mut compressed, 4096, 5, 22);
        writer
            .write_all(&encoded)
            .expect("brotli compressor writer failed to write compressed data to the buffer");
        writer
            .flush()
            .expect("failed to flush brotli compressor writer");
        drop(writer);

        self.to_owned()
            .with_kind(TxKind::Call(Self::STORAGE_ADDRESS))
            .with_input(compressed)
    }
}

/// The underlying network transaction to be sent and an encodable payload
/// containing storage operations. This type implements [`StorageTransactionBuilder`]
/// and is intended to be used just as [`alloy::network::Network::TransactionRequest`].
pub struct StorageTransactionRequest<S: StorageNetwork> {
    payload: <S::StorageTransactionRequest as StorageTransactionBuilder<S>>::Payload,
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
        buf.set_kind(TxKind::Call(
            <S::StorageTransactionRequest as StorageTransactionBuilder<S>>::STORAGE_ADDRESS,
        ));
        buf
    }
}
/// Blanket impl of [`StorageTransactionBuilder`] for [`alloy::network::Ethereum`].
impl StorageTransactionBuilder<Ethereum> for StorageTransactionRequest<Ethereum> {
    const STORAGE_ADDRESS: Address = crate::contract::STORAGE_ADDRESS;

    type Payload = StoragePayload;

    fn payload(&self) -> &Self::Payload {
        &self.payload
    }
    fn payload_mut(&mut self) -> &mut Self::Payload {
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

/// Update a mutable reference to a [`StorageTransactionBuilder`] payload.
///
/// This trait is necessary for mainitaining dynamic compatibility.
pub trait PayloadBuilder<S: StorageNetwork> {
    type Create: Send;
    type Update: Send;
    type Delete: Send;
    type Extend: Send;
    type Chown: Send;

    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn with_creates(&mut self, creates: Vec<Self::Create>);
    fn with_updates(&mut self, updates: Vec<Self::Update>);
    fn with_deletes(&mut self, deletes: Vec<Self::Delete>);
    fn with_extensions(&mut self, extensions: Vec<Self::Extend>);
    fn with_transfers(&mut self, transfers: Vec<Self::Chown>);
}
impl PayloadBuilder<Ethereum> for StoragePayload {
    type Create = Create;
    type Update = Update;
    type Delete = Delete;
    type Extend = Extend;
    type Chown = Chown;

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
    fn with_creates(&mut self, creates: Vec<Self::Create>) {
        self.creates = creates;
    }
    fn with_updates(&mut self, updates: Vec<Self::Update>) {
        self.updates = updates;
    }
    fn with_deletes(&mut self, deletes: Vec<Self::Delete>) {
        self.deletes = deletes;
    }
    fn with_extensions(&mut self, extensions: Vec<Self::Extend>) {
        self.extensions = extensions;
    }
    fn with_transfers(&mut self, transfers: Vec<Self::Chown>) {
        self.transfers = transfers;
    }
}
