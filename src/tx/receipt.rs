use alloy::{
    rpc::types::TransactionReceipt as RawTransactionReceipt, sol_types::SolEventInterface,
};

use crate::contract::{ArkivAbi, Error as ContractError};

pub mod chown;
pub mod create;
pub mod delete;
pub mod extend;
pub mod update;

pub use chown::ChownReceipt;
pub use create::CreateReceipt;
pub use delete::DeleteReceipt;
pub use extend::ExtendReceipt;
pub use update::UpdateReceipt;

#[derive(Debug, Default)]
pub struct TransactionReceipt {
    pub created: Vec<CreateReceipt>,
    pub updated: Vec<UpdateReceipt>,
    pub deleted: Vec<DeleteReceipt>,
    pub extended: Vec<ExtendReceipt>,
    pub transferred: Vec<ChownReceipt>,
}

impl TryFrom<RawTransactionReceipt> for TransactionReceipt {
    type Error = ContractError;

    fn try_from(raw_receipt: RawTransactionReceipt) -> Result<Self, Self::Error> {
        if !raw_receipt.status() {
            return Err(ContractError::BadTransactionStatus {
                receipt: Box::new(raw_receipt),
            });
        }

        let mut receipt = TransactionReceipt::default();
        raw_receipt
            .into_primitives_receipt()
            .logs()
            .iter()
            .try_for_each(|log| -> Result<(), ContractError> {
                ArkivAbi::ArkivAbiEvents::decode_log(log)
                    .map(|parsed| match parsed.data {
                        ArkivAbi::ArkivAbiEvents::ArkivEntityCreated(data) => {
                            receipt.created.push(CreateReceipt::from(data))
                        }
                        ArkivAbi::ArkivAbiEvents::ArkivEntityUpdated(data) => {
                            receipt.updated.push(UpdateReceipt::from(data))
                        }
                        ArkivAbi::ArkivAbiEvents::ArkivEntityDeleted(data) => {
                            receipt.deleted.push(DeleteReceipt::from(data))
                        }
                        ArkivAbi::ArkivAbiEvents::ArkivEntityBTLExtended(data) => {
                            receipt.extended.push(ExtendReceipt::from(data))
                        }
                        ArkivAbi::ArkivAbiEvents::ArkivEntityOwnerChanged(data) => {
                            receipt.transferred.push(ChownReceipt::from(data))
                        }
                        ArkivAbi::ArkivAbiEvents::ArkivEntityExpired(_) => unreachable!("House keeping transactions are automatic. Receipts are only emitted for transactions sent via program.")
                    })
                    .map_err(ContractError::from)
            })?;

        Ok(receipt)
    }
}
