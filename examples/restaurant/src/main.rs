//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

mod proto;
mod service;

use proto::restaurant::restaurant_server::RestaurantServer;
use service::RestaurantManager;
use tonic::transport::{Error, Server};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Main Function
//++++++++++++++++++++++++++++++++++++++++++++++++++//

#[tokio::main]
async fn main() -> Result<(), Error> {
    let service = RestaurantManager::new();

    let addr = "0.0.0.0:3160".parse().unwrap();

    println!("Restaurant was opened");

    Server::builder()
        .add_service(RestaurantServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
