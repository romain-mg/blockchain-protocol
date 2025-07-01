use anyhow::Result;
use async_trait::async_trait;
use blockchain_core::{P2P_SERVER_ADDR, SECONDARY_P2P_SERVER_ADDR, log, rpc::p2p::*};
use tonic::{Request, Response, Status, transport::Server};

pub struct P2pServer {}

#[async_trait]
impl p2p_server::P2p for P2pServer {
    async fn p2p_example(
        &self,
        request: Request<P2pExampleRequest>,
    ) -> Result<Response<P2pExampleReply>, Status> {
        log::info!("received example request");
        Ok(Response::new(P2pExampleReply { output: 0 }))
    }
}

pub async fn start() -> Result<()> {
    let svc = p2p_server::P2pServer::new(P2pServer {});
    log::info!("Starting p2p server on {}", P2P_SERVER_ADDR);
    Server::builder()
        .add_service(svc)
        .serve(P2P_SERVER_ADDR.parse().unwrap())
        .await?;
    Ok(())
}

// For testing purposes
pub async fn start_secondary() -> Result<()> {
    let svc = p2p_server::P2pServer::new(P2pServer {});
    log::info!(
        "Starting secondary p2p server on {}",
        SECONDARY_P2P_SERVER_ADDR
    );
    Server::builder()
        .add_service(svc)
        .serve(SECONDARY_P2P_SERVER_ADDR.parse().unwrap())
        .await?;
    Ok(())
}
