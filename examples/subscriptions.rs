//! Spawn an ephemeral arkiv node and log events emitted by the storage contract.

use std::{
    error, io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time,
};

use alloy::{
    primitives::U256,
    signers::{Signer, local::LocalSigner},
};
use arkiv_sdk::{
    Attribute, BlocksToLive, Provider, ProviderBuilder, StorageProvider,
    entity::NumericAttribute,
    node_bindings::Arkiv,
    ops::{Create, Extend, WithAttribute},
    rpc::types::ArkivEvent,
    tx::TransactionReceipt,
};

const FAUCET_FUNDS: U256 = U256::from_limbs([0, 100, 0, 0]);
const FOUR_SECONDS: BlocksToLive = BlocksToLive::new(8);
const PAYLOAD: &str = "Hello, world!";
const CONTENT_TYPE: &str = "plain/text";
const TIMESTAMP: Attribute<&str, &str> = Attribute::new("timestamp", "1970-01-01T00:00:00Z");
const VERSION: Attribute<&str, u8> = Attribute::new("version", 1);
const BLOCK_PERIOD: time::Duration = time::Duration::from_secs(2);
const TIMEOUT: time::Duration = time::Duration::from_secs(120);

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(io::stdout)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not set up global logger");

    let arkiv = Arkiv::default()
        .dev_period(BLOCK_PERIOD) // produce one block every 2s
        .spawn()?;

    let entity_expired = Arc::new(AtomicBool::new(false));
    tokio::spawn({
        let chain_id = arkiv.networkid();
        let ws_url = arkiv.ws_endpoint_url();
        let entity_expired = entity_expired.clone();
        async move {
            use alloy::providers::WsConnect;
            use futures::StreamExt;

            let ws_provider = ProviderBuilder::new()
                .with_chain_id(chain_id)
                .connect_ws(WsConnect::new(ws_url.clone()))
                .await
                .expect("websocket provider failed to connect to websocket")
                .erased();

            let mut stream = ws_provider
                .subscribe_storage_events(|f| f)
                .await
                .expect("failed to get storage event stream handle");
            tracing::info!("websocket is listening for events ->> {}", ws_url);
            while let Some(event) = stream.next().await {
                match event {
                    Ok(event) => match event {
                        ArkivEvent::EntityExpired { .. } => {
                            tracing::info!(?event);
                            entity_expired.store(true, Ordering::SeqCst);
                        }
                        _ => tracing::info!(?event),
                    },
                    Err(err) => tracing::error!("failed to parse event log: {err:?}"),
                }
            }
        }
    });

    tokio::time::timeout(TIMEOUT, async {
        run(arkiv.networkid(), arkiv.endpoint_url(), entity_expired)
            .await
            .expect("failed during main process")
    })
    .await?;

    Ok(())
}

async fn run(
    networkid: u64,
    endpoint_url: reqwest::Url,
    entity_expired: Arc<AtomicBool>,
) -> Result<(), Box<dyn error::Error>> {
    let http_provider = ProviderBuilder::new()
        .with_chain_id(networkid)
        .connect_http(endpoint_url.clone())
        .erased();

    tracing::info!("generating fee history, this could take a while...");

    arkiv_sdk::utils::generate_fee_history(&http_provider, endpoint_url.clone()).await;

    tracing::info!("successfully generated fee history");

    let signer = LocalSigner::random().with_chain_id(Some(networkid));
    let address = signer.address();
    let wallet_provider = ProviderBuilder::new()
        .with_chain_id(networkid)
        .fetch_chain_id()
        .wallet(signer)
        .connect_http(endpoint_url)
        .erased();
    arkiv_sdk::utils::fund_account(&http_provider, address, FAUCET_FUNDS).await;

    let pending_create = wallet_provider
        .create_entities(vec![
            Create::new()
                .btl(FOUR_SECONDS)
                .payload(PAYLOAD)
                .content_type(CONTENT_TYPE)
                .with_attribute(TIMESTAMP.map_into::<String>())
                .with_attribute(VERSION.map(NumericAttribute::from))
                .build()?,
        ])
        .await?;
    let receipt = pending_create.get_receipt().await?;

    let TransactionReceipt { created, .. } = receipt.try_into()?;
    let entity_key = created[0].entity_key;

    let extend_receipt = wallet_provider
        .extend_entities(vec![Extend::new(entity_key, time::Duration::from_secs(2))])
        .await?
        .get_receipt()
        .await?;
    assert!(extend_receipt.status());

    while !entity_expired.load(Ordering::SeqCst) {
        tokio::time::sleep(BLOCK_PERIOD).await;
    }

    Ok(())
}
