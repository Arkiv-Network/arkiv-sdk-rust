use alloy::{network::EthereumWallet, primitives::U256, signers::Signer};
use arkiv_sdk::{PrivateKeySigner, Provider, ProviderBuilder, node_bindings::Arkiv};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let arkiv = Arkiv::default().spawn()?;
    eprintln!("keystore-signer: arkiv node pid: {}", arkiv.id());

    let mut signer = PrivateKeySigner::random();
    signer.set_chain_id(Some(arkiv.networkid()));
    let wallet = EthereumWallet::from(signer.clone());

    let ro_client = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .with_simple_nonce_management()
        .with_gas_estimation()
        .connect_http(arkiv.endpoint_url())
        .erased();
    let client = ProviderBuilder::new()
        .with_chain_id(arkiv.networkid())
        .with_simple_nonce_management()
        .with_gas_estimation()
        .wallet(wallet)
        .connect_http(arkiv.endpoint_url())
        .erased();

    let requested_funds = 100;
    let receipt =
        arkiv_sdk::utils::fund_account(&ro_client, signer.address(), requested_funds).await;
    eprintln!("keystore-signer: {receipt:?}");
    let balance = client.get_balance(signer.address()).await.unwrap();

    assert_eq!(
        bigdecimal::BigDecimal::from(requested_funds),
        arkiv_sdk::utils::wei_to_eth(U256::from(balance)),
        "requested funds do not match current balance"
    );

    Ok(())
}
