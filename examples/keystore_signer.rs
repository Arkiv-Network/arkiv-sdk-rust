//! Spawn an ephemeral arkiv node, add faucet funds to an account and send storage transactions.

use std::time::Duration;

use alloy::{primitives::U256, signers::Signer};
use arkiv_sdk::{
    PrivateKeySigner, Provider, ProviderBuilder, StorageProvider, node_bindings::Arkiv,
    ops::Create, tx::StorageTransactionBuilder,
};

const FAUCET_FUNDS: U256 = U256::from_limbs([100, 0, 0, 0]);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arkiv = Arkiv::default().spawn()?;

    eprintln!("keystore-signer: arkiv node pid: {}", arkiv.id());

    let node_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .connect_http(arkiv.endpoint_url())
        .erased();

    let signer = PrivateKeySigner::random().with_chain_id(Some(arkiv.networkid()));
    let address = signer.address();
    let wallet_provider = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .wallet(signer)
        .connect_http(arkiv.endpoint_url())
        .erased();

    let receipt = arkiv_sdk::utils::fund_account(&node_provider, address, FAUCET_FUNDS).await;

    eprintln!("keystore-signer: {receipt:?}");

    assert_eq!(
        FAUCET_FUNDS,
        wallet_provider.get_balance(address).await?,
        "keystore-signer: requested funds do not match current balance"
    );

    let tx = wallet_provider.storage_transaction().create_entities(vec![
        Create::new()
            .btl(Duration::from_secs(10))
            .payload("Hello, world!")
            .content_type("plain/text")
            .build()?,
    ]);

    let receipt = wallet_provider
        .send_storage_transaction(tx)
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();

    // TODO: this currently returns no receipt for some reason.
    dbg!(&receipt);

    Ok(())
}
