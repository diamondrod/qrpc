//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use super::proto::example_service::{Acceptance, Expense, History, Order, Total};
use super::proto::example_service::restaurant_server::Restaurant;
use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Global Variables
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Prices for each menu.
const PRICES: [f32; 9] = [0_f32, 7.5, 6.8, 4.0, 9.25, 10.0, 4.25, 3.3, 2.0];

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Structs
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Manager handling orders and cacher service.
pub(crate) struct RestaurantManager{
  /// Order history for each table.
  tables: RwLock<HashMap<i32, Vec<History>>>,
  /// Capacity of a kitchen to accept items at once.
  capacity: u8
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

#[async_trait]
impl Restaurant for RestaurantManager{
    async fn submit(
      &self,
      request: Request<Order>,
    ) -> Result<Response<Acceptance>, Status>{
      let order = request.into_inner();
      if (self.capacity as usize) < order.items.len(){
        // Out of capacity
        Ok(Response::new(Acceptance{
          accepted: false,
          reason: String::from("too many items. must be less than 10")
        }))
      }
      else{
        let time = order.ordered_time;
        // Build a map from item to unit, and then to history
        let history = order.items.into_iter().fold(HashMap::new(), |mut map, item|{
          if let Some(record) = map.get_mut(&item){
            *record += 1;
          }
          else{
            map.insert(item, 1);
          }
          map
        }).into_iter().map(|(k, v)|{
          History{
            time: time.clone(),
            item: k,
            unit: v,
            price: PRICES[k as usize]
          }
        }).collect::<Vec<History>>();

        // Update internal table
        let mut tables = self.tables.write().await;
        tables.insert(order.table, history);

        Ok(Response::new(Acceptance{
          accepted: true,
          reason: String::new()
        }))
      }
      
    }

    async fn finish(
        &self,
        request: Request<Expense>,
    ) -> Result<Response<Total>, Status>{
      let expense = request.into_inner();
      let mut tables = self.tables.write().await;
      if let Some(history) = tables.remove(&expense.table){
        // History exists for the table
        let total = history.iter().map(|h| h.unit as f32 * h.price).sum();
        Ok(Response::new(Total{
          history,
          total
        }))
      }
      else{
        Err(Status::internal(format!("no order for the table id: {}", expense.table)))
      }
    }
}

impl RestaurantManager{
  pub(crate) fn new() -> Self{
    Self{
      tables: RwLock::new(HashMap::new()),
      capacity: 9
    }
  }
}
