use tonic::{transport::Server, Request, Response, Status};

pub mod api;
pub mod service;

use service::GrpcRearwardService;
use api::rearward_api::server::RearwardServiceServer;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let rearward_grpc = GrpcRearwardService::default();

    Server::builder()
        .serve(addr, RearwardServiceServer::new(rearward_grpc)).await?;
    Ok(())
}
