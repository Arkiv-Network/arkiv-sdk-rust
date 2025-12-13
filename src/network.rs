use alloy::network::{Ethereum, Network};

pub trait StorageNetwork: Network {
    type StorageTransactionRequest: crate::tx::StorageTransactionBuilder<Self> + Send;
    type Create: Send;
    type Update: Send;
    type Delete: Send;
    type Extend: Send;
    type Chown: Send;
    type Payload: Default + alloy_rlp::Encodable + crate::tx::PayloadBuilder<Self> + Send;
}
impl StorageNetwork for Ethereum {
    type StorageTransactionRequest = crate::tx::StorageTransactionRequest<Self>;
    type Create = crate::tx::ops::create::Create;
    type Update = crate::tx::ops::update::Update;
    type Delete = crate::tx::ops::delete::Delete;
    type Extend = crate::tx::ops::extend::Extend;
    type Chown = crate::tx::ops::chown::Chown;
    type Payload = crate::tx::StoragePayload;
}
