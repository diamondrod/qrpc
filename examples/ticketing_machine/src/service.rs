//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use super::proto::q::Symbol;
use super::proto::ticket::ticketing_machine_server::TicketingMachine;
use super::proto::ticket::{Application, AvailableSeats, Cancelled, Class, Processed, TicketInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Structs
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Seat in a studium.
struct Seat {
    /// Name of a customer who reserved a seat.
    name: Option<String>,
    /// Class of a seat.
    class: Class,
}

/// Ticketing machine.
pub(crate) struct Machine {
    /// Inventory of seats tagged with ID.
    inventory: RwLock<HashMap<String, Seat>>,
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% Seat %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

impl Seat {
    /// Default stand seat.
    fn stand() -> Self {
        Self {
            name: None,
            class: Class::Stand,
        }
    }

    /// Default arena seat.
    fn arena() -> Self {
        Self {
            name: None,
            class: Class::Arena,
        }
    }

    /// Default VIP seat.
    fn vip() -> Self {
        Self {
            name: None,
            class: Class::Vip,
        }
    }
}

//%% Machine %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

impl Machine {
    pub(crate) fn new() -> Self {
        let inventory = RwLock::new(HashMap::from([
            (String::from("s1"), Seat::stand()),
            (String::from("s2"), Seat::stand()),
            (String::from("s3"), Seat::stand()),
            (String::from("a1"), Seat::arena()),
            (String::from("a2"), Seat::arena()),
            (String::from("v1"), Seat::vip()),
        ]));

        Self { inventory }
    }
}

#[async_trait]
impl TicketingMachine for Machine {
    async fn reserve(&self, request: Request<Application>) -> Result<Response<Processed>, Status> {
        let application = request.into_inner();
        let mut inventory = self.inventory.write().await;
        let mut available = inventory
            .iter_mut()
            .filter(|(_, seat)| (seat.class as i32 == application.class) && seat.name.is_none())
            .take(application.number as usize)
            .collect::<HashMap<&String, &mut Seat>>();
        if available.len() < application.number as usize {
            Ok(Response::new(Processed::failure(
                available.len(),
                application.number,
            )))
        } else {
            let seats = available
                .iter_mut()
                .map(|(id, seat)| {
                    seat.name = Some(application.name.clone());
                    Symbol {
                        symbol: id.to_string(),
                    }
                })
                .collect::<Vec<Symbol>>();
            Ok(Response::new(Processed::success(
                application.name.clone(),
                seats,
                application.date,
            )))
        }
    }

    async fn cancel(&self, request: Request<TicketInfo>) -> Result<Response<Cancelled>, Status> {
        let info = request.into_inner();
        let mut inventory = self.inventory.write().await;
        let mut cancelled = Vec::new();
        let mut error = Vec::new();
        info.seats.iter().for_each(|seat| {
            if let Some(reserved) = inventory.get_mut(&seat.symbol) {
                // Seat exists
                if reserved.name.is_some() && *reserved.name.as_ref().unwrap() == info.name {
                    // Reserved seat for the customer
                    reserved.name = None;
                    cancelled.push(seat.symbol.clone());
                } else {
                    // Non-existing seat or not reserved by the customer
                    error.push(seat.symbol.clone());
                }
            } else {
                error.push(seat.symbol.clone());
            }
        });

        let mut message = String::new();
        if cancelled.len() != 0 {
            message = [message, format!("{} were cancelled.", cancelled.join(","))].concat();
        }
        if error.len() != 0 {
            message = [
                message,
                format!("{} could not be cancelled.", error.join(",")),
            ]
            .concat();
        }
        Ok(Response::new(Cancelled { message }))
    }

    async fn get_available_seats(
        &self,
        _request: Request<()>,
    ) -> Result<Response<AvailableSeats>, Status> {
        let inventory = self.inventory.read().await;
        let seats = inventory
            .iter()
            .fold(HashMap::new(), |mut map, (_, seat)| {
                match seat.class {
                    Class::NoPreference => (),
                    Class::Stand | Class::Arena | Class::Vip => {
                        let class = seat.class.to_string();
                        if let Some(counter) = map.get_mut(&class) {
                            *counter += 1;
                        } else {
                            map.insert(class, 1);
                        }
                    }
                }
                map
            });
        Ok(Response::new(AvailableSeats { inventory: seats }))
    }
}
