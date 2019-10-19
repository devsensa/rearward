use tonic::{transport::Server, Request, Response, Status};

pub mod lib;

use lib::hello_world::{server::{Greeter,GreeterServer},HelloReply,HelloRequest};

#[derive(Default)]
pub struct GrpcGreeter {
    data: String
}

#[tonic::async_trait]
impl Greeter for GrpcGreeter {
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let string = &self.data;

        println!("My data: {:?}", string);

        let reply = HelloReply {
            message: "Zomg, it works!".into(),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greet = GrpcGreeter::default();

    Server::builder().serve(addr, GreeterServer::new(greet)).await?;
    Ok(())
}
