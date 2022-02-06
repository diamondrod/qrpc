mod proto;
mod service;

use proto::example_service::restaurant_server::RestaurantServer;
use service::RestaurantManager;
use tonic::transport::{Error, Server};

#[tokio::main]
async fn main() -> Result<(), Error>{
  let service = RestaurantManager::new();
  
  let addr = "0.0.0.0:3160".parse().unwrap();

  Server::builder()
      .add_service(RestaurantServer::new(service))
      .serve(addr)
      .await?;

  Ok(())
}
