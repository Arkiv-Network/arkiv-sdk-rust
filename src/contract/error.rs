#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Transaction `{}` failed: {:?}", receipt.transaction_hash, receipt)]
    BadTransactionStatus {
        receipt: Box<alloy::rpc::types::TransactionReceipt>,
    },

    #[error(transparent)]
    AbiError(#[from] alloy::sol_types::Error),
}
