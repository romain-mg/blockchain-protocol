use anyhow::Result;
use blockchain_core::{P2P_SERVER_ADDR, rpc::p2p::*};
use tonic::transport::Channel;

async fn connect() -> Result<p2p_client::P2pClient<Channel>> {
    Ok(p2p_client::P2pClient::connect(format!("http://{}", P2P_SERVER_ADDR)).await?)
}

pub async fn example(input: u32) -> Result<u32> {
    let mut client = connect().await?;
    Ok(client
        .p2p_example(P2pExampleRequest { input })
        .await?
        .into_inner()
        .output)
}
