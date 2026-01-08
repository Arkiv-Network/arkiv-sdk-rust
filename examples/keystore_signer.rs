use arkiv_sdk::{
    PrivateKeySigner, Provider, ProviderBuilder,
    node_bindings::{Arkiv, Tag},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let arkiv = Arkiv::default()
        .fetch_tag(Tag::Latest)
        .download_dir("../../")
        .spawn()?;
    println!("arkiv node pid: {}", arkiv.id());

    // TODO: Add a test keystore
    let wallet = PrivateKeySigner::random();
    let _client = ProviderBuilder::new()
        .with_simple_nonce_management()
        .wallet(wallet)
        .connect_http(arkiv.endpoint_url())
        .erased();

    Ok(())
}
