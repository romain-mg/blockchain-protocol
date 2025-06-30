use anyhow::Result;
use async_trait::async_trait;
use blockchain_core::{SERVER_ADDR, rpc::p2p::*};
use tonic::{Request, Response, Status, transport::Server};

pub struct P2pServer {}

#[async_trait]
impl p2p_server::P2p for P2pServer {
    async fn p2p_example(
        &self,
        request: Request<P2pExampleRequest>,
    ) -> Result<Response<P2pExampleReply>, Status> {
        println!("received example request");
        Ok(Response::new(P2pExampleReply { output: 0 }))
    }
}

pub async fn start() -> Result<()> {
    let svc = p2p_server::P2pServer::new(P2pServer {});
    Server::builder()
        .add_service(svc)
        .serve(SERVER_ADDR.parse().unwrap())
        .await?;
    Ok(())
}
