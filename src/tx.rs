use std::ops::{Deref, DerefMut};

use alloy::{
    network::{Network, TransactionBuilder},
    primitives::{Address, TxKind},
};
use alloy_rlp::{Encodable, RlpDecodable, RlpEncodable};

pub mod ops;
pub mod receipt;

use ops::{chown::Chown, create::Create, delete::Delete, extend::Extend, update::Update};

/// Extension trait for adding storage functionality to [`alloy::network::TransactionBuilder`].
pub trait StorageTransactionBuilder<N: Network>:
    Deref<Target = N::TransactionRequest> + DerefMut + Sized
{
    /// The address of the storage contract.
    const STORAGE_ADDRESS: Address;

    /// Access the underlying payload
    fn payload(&self) -> &StoragePayload;
    fn payload_mut(&mut self) -> &mut StoragePayload;

    /// Add a list of [`Create`] operations to the encodable transaction inputs.
    fn create_entities<C: Into<Vec<Create>>>(mut self, creates: C) -> Self {
        self.payload_mut().creates = creates.into();
        self
    }

    /// Add a list of [`Update`] operations to the encodable transaction inputs.
    fn update_entities<U: Into<Vec<Update>>>(mut self, updates: U) -> Self {
        self.payload_mut().updates = updates.into();
        self
    }

    /// Add a list of [`Delete`] operations to the encodable transaction inputs.
    fn delete_entities<D: Into<Vec<Delete>>>(mut self, deletes: D) -> Self {
        self.payload_mut().deletes = deletes.into();
        self
    }

    /// Add a list of [`Extend`] operations to the encodable transaction inputs.
    fn extend_entities<E: Into<Vec<Extend>>>(mut self, extensions: E) -> Self {
        self.payload_mut().extensions = extensions.into();
        self
    }

    /// Add a list of [`Chown`] operations to the encodable transaction inputs.
    fn transfer_entities<T: Into<Vec<Chown>>>(mut self, transfers: T) -> Self {
        self.payload_mut().transfers = transfers.into();
        self
    }

    /// Encode the storage payload and set the transaction input and [`alloy::primitives::TxKind`],
    /// returning the inner [`alloy::network::Network::TransactionRequest`].
    fn into_request(mut self) -> N::TransactionRequest {
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
pub struct StorageTransactionRequest<N: Network> {
    payload: StoragePayload,
    request: N::TransactionRequest,
}
impl<N: Network> Deref for StorageTransactionRequest<N> {
    type Target = N::TransactionRequest;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}
impl<N: Network> DerefMut for StorageTransactionRequest<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.request
    }
}
impl<N: Network> Default for StorageTransactionRequest<N> {
    fn default() -> Self {
        let mut buf = Self {
            payload: Default::default(),
            request: Default::default(),
        };
        buf.set_kind(TxKind::Call(Self::STORAGE_ADDRESS));
        buf
    }
}
impl<N: Network> StorageTransactionBuilder<N> for StorageTransactionRequest<N> {
    const STORAGE_ADDRESS: Address = crate::eth::STORAGE_ADDRESS;
    fn payload(&self) -> &StoragePayload {
        &self.payload
    }
    fn payload_mut(&mut self) -> &mut StoragePayload {
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
impl StoragePayload {
    pub fn is_empty(&self) -> bool {
        self.creates.is_empty()
            && self.updates.is_empty()
            && self.deletes.is_empty()
            && self.extensions.is_empty()
            && self.transfers.is_empty()
    }
    pub fn len(&self) -> usize {
        self.creates.len()
            + self.updates.len()
            + self.deletes.len()
            + self.extensions.len()
            + self.transfers.len()
    }
}

impl<N: Network> From<Vec<Create>> for StorageTransactionRequest<N> {
    fn from(creates: Vec<Create>) -> Self {
        StorageTransactionRequest {
            payload: StoragePayload {
                creates,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
impl<N: Network> From<Vec<Update>> for StorageTransactionRequest<N> {
    fn from(updates: Vec<Update>) -> Self {
        StorageTransactionRequest {
            payload: StoragePayload {
                updates,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
impl<N: Network> From<Vec<Delete>> for StorageTransactionRequest<N> {
    fn from(deletes: Vec<Delete>) -> Self {
        StorageTransactionRequest {
            payload: StoragePayload {
                deletes,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
impl<N: Network> From<Vec<Extend>> for StorageTransactionRequest<N> {
    fn from(extensions: Vec<Extend>) -> Self {
        StorageTransactionRequest {
            payload: StoragePayload {
                extensions,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
