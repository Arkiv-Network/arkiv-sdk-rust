use std::{io, path};

use arkiv_sdk::node_bindings::Arkiv;

#[tokio::main]
async fn main() -> io::Result<()> {
    let node = Arkiv::default()
        .fetch_latest(Some(path::PathBuf::from("../../")))
        .spawn()?;
    println!("arkiv node pid: {}", node.id());

    Ok(())
}
