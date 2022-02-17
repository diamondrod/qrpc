//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

mod proto;
mod service;

use proto::ticket::ticketing_machine_server::TicketingMachineServer;
use service::Machine;
use tonic::transport::{Error, Server};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Main Function
//++++++++++++++++++++++++++++++++++++++++++++++++++//

#[tokio::main]
async fn main() -> Result<(), Error> {
    let service = Machine::new();

    let addr = "0.0.0.0:3130".parse().unwrap();

    println!("Ticketing machine was booted");

    Server::builder()
        .add_service(TicketingMachineServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
