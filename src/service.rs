use super::api::rearward_api::server::{RearwardService, RearwardServiceServer};
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct GrpcRearwardService;

#[tonic::async_trait]
impl RearwardService for GrpcRearwardService {
    async fn healthcheck(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}
