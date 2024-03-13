mod server;
mod proto {
    tonic::include_proto!("piston");
}

use std::error::Error;

use proto::piston_server::PistonServer;
use server::PistonService;
use tonic::transport::Server;
use tonic_reflection::server::Builder;

const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/piston_descriptor.bin"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let addr = std::env::var("SERVE_ADDR")
        .expect("SERVE_ADDR must be set")
        .parse()?;

    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;
    let piston_service = PistonService;

    println!("Piston server listening on {}", addr);

    Server::builder()
        .add_service(reflection_service)
        .add_service(PistonServer::new(piston_service))
        .serve(addr)
        .await?;

    Ok(())
}
