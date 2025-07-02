use anyhow::Result;
use async_trait::async_trait;
use blockchain_core::{SECONDARY_SERVER_ADDR, SERVER_ADDR, log, rpc::client_to_node::*};
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
    let svc = client_to_node_server::ClientToNodeServer::new(ClientToNodeServer {});
    log::info!("Starting client to node server on {}", SERVER_ADDR);
    Server::builder()
        .add_service(svc)
        .serve(SERVER_ADDR.parse().unwrap())
        .await?;
    Ok(())
}

// For testing purposes
pub async fn start_secondary() -> Result<()> {
    let svc = client_to_node_server::ClientToNodeServer::new(ClientToNodeServer {});
    log::info!(
        "Starting secondary client to node server on {}",
        SECONDARY_SERVER_ADDR
    );
    Server::builder()
        .add_service(svc)
        .serve(SECONDARY_SERVER_ADDR.parse().unwrap())
        .await?;
    Ok(())
}
