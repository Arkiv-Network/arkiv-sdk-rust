use arkiv_sdk::node_bindings::{Arkiv, Tag};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let node = Arkiv::default()
        .fetch_tag(Tag::Latest)
        .download_dir("../../")
        .spawn()?;
    println!("arkiv node pid: {}", node.id());

    Ok(())
}
