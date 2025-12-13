use alloy::{
    rpc::types::TransactionReceipt as RawTransactionReceipt, sol_types::SolEventInterface,
};

pub mod chown;
pub mod create;
pub mod delete;
pub mod extend;
pub mod update;

use chown::ChownReceipt;
use create::CreateReceipt;
use delete::DeleteReceipt;
use extend::ExtendReceipt;
use update::UpdateReceipt;

use crate::eth::ArkivAbi;

#[derive(Debug, Default)]
pub struct TransactionReceipt {
    pub created: Vec<CreateReceipt>,
    pub updated: Vec<UpdateReceipt>,
    pub deleted: Vec<DeleteReceipt>,
    pub extended: Vec<ExtendReceipt>,
    pub transferred: Vec<ChownReceipt>,
}

impl TryFrom<RawTransactionReceipt> for TransactionReceipt {
    type Error = crate::eth::Error;

    fn try_from(raw_receipt: RawTransactionReceipt) -> Result<Self, Self::Error> {
        if !raw_receipt.status() {
            return Err(Self::Error::TransactionReceiptError(format!(
                "Transaction {} failed: {:?}",
                raw_receipt.transaction_hash, raw_receipt
            )));
        }

        let mut receipt = TransactionReceipt::default();
        raw_receipt
            .into_primitives_receipt()
            .logs()
            .iter()
            .try_for_each(|log| {
                let parsed = ArkivAbi::ArkivAbiEvents::decode_log(&log).map_err(|e| {
                    // TODO: Fix error types
                    Self::Error::UnexpectedLogDataError(format!("Error decoding event log: {e}"))
                })?;
                match parsed.data {
                    ArkivAbi::ArkivAbiEvents::EntityCreated(data) => {
                        receipt.created.push(CreateReceipt::from(data))
                    }
                    ArkivAbi::ArkivAbiEvents::EntityUpdated(data) => {
                        receipt.updated.push(UpdateReceipt::from(data))
                    }
                    ArkivAbi::ArkivAbiEvents::EntityDeleted(data) => {
                        receipt.deleted.push(DeleteReceipt::from(data))
                    }
                    ArkivAbi::ArkivAbiEvents::EntityExtended(data) => {
                        receipt.extended.push(ExtendReceipt::from(data))
                    }
                    ArkivAbi::ArkivAbiEvents::EntityTransferred(data) => {
                        receipt.transferred.push(ChownReceipt::from(data))
                    }
                }
                Ok(())
            })?;

        Ok(receipt)
    }
}
