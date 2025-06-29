use anyhow::Result;
use async_trait::async_trait;
use blockchain_core::{SERVER_ADDR, rpc::client_to_node::*};
use tonic::{Request, Response, Status, transport::Server};

pub struct ClientToNodeServer {}

#[async_trait]
impl client_to_node_server::ClientToNode for ClientToNodeServer {
    async fn client_to_node_example(
        &self,
        request: Request<ClientToNodeExampleRequest>,
    ) -> Result<Response<ClientToNodeExampleReply>, Status> {
        println!("received example request");
        Ok(Response::new(ClientToNodeExampleReply { output: 0 }))
    }
}

pub async fn start() -> Result<()> {
    let svc = client_to_node_server::ClientToNodeServer::new(ClientToNodeServer {});
    Server::builder()
        .add_service(svc)
        .serve(SERVER_ADDR.parse().unwrap())
        .await?;
    Ok(())
}
