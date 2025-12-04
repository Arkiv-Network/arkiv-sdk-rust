use alloy_rlp::{Encodable, RlpDecodable, RlpEncodable};
use bon::bon;

pub mod ops;
pub mod receipt;

use ops::{create::Create, delete::Delete, extend::Extend, update::Update};

/// Type representing a transaction in GolemBase, including creates, updates, deletes, and extensions.
/// Used as the main payload for submitting entity changes to the chain.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub encodable: EncodableTransaction,
    pub gas_limit: Option<u64>,
    pub max_priority_fee_per_gas: Option<u128>,
    pub max_fee_per_gas: Option<u128>,
}
#[bon]
impl Transaction {
    #[builder]
    pub fn builder(
        creates: Option<Vec<Create>>,
        updates: Option<Vec<Update>>,
        deletes: Option<Vec<Delete>>,
        extensions: Option<Vec<Extend>>,
        gas_limit: Option<u64>,
        max_priority_fee_per_gas: Option<u128>,
        max_fee_per_gas: Option<u128>,
    ) -> Self {
        Self {
            encodable: EncodableTransaction {
                creates: creates.unwrap_or_default(),
                updates: updates.unwrap_or_default(),
                deletes: deletes.unwrap_or_default(),
                extensions: extensions.unwrap_or_default(),
            },
            gas_limit,
            max_priority_fee_per_gas,
            max_fee_per_gas,
        }
    }
}
impl Transaction {
    /// Returns the RLP-encoded bytes of the transaction.
    /// Useful for submitting the transaction to the chain.
    pub fn encoded(&self) -> Vec<u8> {
        let mut encoded = Vec::new();
        self.encodable.encode(&mut encoded);
        encoded
    }
}

/// A transaction that can be encoded in RLP
#[derive(Debug, Clone, Default, RlpEncodable, RlpDecodable)]
pub struct EncodableTransaction {
    // TODO: Add chown
    /// A list of [`Create`] operations.
    pub creates: Vec<Create>,
    /// A list of [`Update`] operations.
    pub updates: Vec<Update>,
    /// A list of [`Delete`] operations.
    pub deletes: Vec<Delete>,
    /// A list of [`Extend`] operations.
    pub extensions: Vec<Extend>,
}
