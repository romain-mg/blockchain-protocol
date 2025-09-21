use anyhow::Result;
use async_trait::async_trait;
use blockchain_core::{log, rpc::client_to_node::*};
use std::env;
use tonic::{Request, Response, Status, transport::Server};

pub struct ClientToNodeServer {}

#[async_trait]
impl client_to_node_server::ClientToNode for ClientToNodeServer {
    async fn client_to_node_example(
        &self,
        request: Request<ClientToNodeExampleRequest>,
    ) -> Result<Response<ClientToNodeExampleReply>, Status> {
        log::info!("received example request");
        Ok(Response::new(ClientToNodeExampleReply { output: 0 }))
    }
}

pub async fn start() -> Result<()> {
    let server_addr = env::var("SERVER_ADDR").expect("server address not found");
    let svc = client_to_node_server::ClientToNodeServer::new(ClientToNodeServer {});
    log::info!("Starting client to node server on {}", server_addr);
    Server::builder()
        .add_service(svc)
        .serve(server_addr.parse().expect("Invalid server address"))
        .await?;
    Ok(())
}

// For testing purposes
pub async fn start_secondary() -> Result<()> {
    let secondary_server_addr =
        env::var("SECONDARY_SERVER_ADDR").expect("secondary server address not found");
    let svc = client_to_node_server::ClientToNodeServer::new(ClientToNodeServer {});
    log::info!(
        "Starting secondary client to node server on {}",
        secondary_server_addr
    );
    Server::builder()
        .add_service(svc)
        .serve(secondary_server_addr.parse().expect("Invalid secondary server address"))
        .await?;
    Ok(())
}
