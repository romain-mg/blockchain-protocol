use anyhow::Result;
use blockchain_core::{log, rpc::client_to_node::*};
use std::env;
use tonic::transport::Channel;

async fn connect() -> Result<client_to_node_client::ClientToNodeClient<Channel>> {
    let server_addr = env::var("SERVER_ADDR").expect("server address not found");
    Ok(
        client_to_node_client::ClientToNodeClient::connect(format!("http://{}", server_addr))
            .await?,
    )
}

pub async fn example(input: u32) -> Result<u32> {
    let mut client = connect().await?;
    let response = client
        .client_to_node_example(ClientToNodeExampleRequest { input })
        .await?
        .into_inner()
        .output;
    Ok(response)
}
