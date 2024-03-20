mod foundry;

use iota_sdk::client::Result;

use crate::foundry::NodeData;

#[tokio::main]
async fn main() -> Result<()> {
    let mainnet = NodeData::collect("https://api.stardust-mainnet.iotaledger.net").await?;
    let shimmer = NodeData::collect("https://api.shimmer.network").await?;

    println!("{:#?}", mainnet.stats());
    println!("{:#?}", shimmer.stats());

    Ok(())
}
