use anyhow::Result;
use blockchain_core::{SERVER_ADDR, rpc::client_to_node::*};
use tonic::transport::Channel;

async fn connect() -> Result<client_to_node_client::ClientToNodeClient<Channel>> {
    Ok(
        client_to_node_client::ClientToNodeClient::connect(format!("http://{}", SERVER_ADDR))
            .await?,
    )
}

pub async fn example(input: u32) -> Result<u32> {
    let mut client = connect().await?;
    Ok(client
        .client_to_node_example(ClientToNodeExampleRequest { input })
        .await?
        .into_inner()
        .output)
}
