//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use super::proto::restaurant::restaurant_server::Restaurant;
use super::proto::restaurant::{Acceptance, Expense, History, Order, Total};
use async_trait::async_trait;
use std::collections::HashMap;
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
pub(crate) struct RestaurantManager {
    /// Order history for each table.
    tables: RwLock<HashMap<i32, Vec<History>>>,
    /// Capacity of a kitchen to accept items at once.
    capacity: u8,
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

#[async_trait]
impl Restaurant for RestaurantManager {
    async fn submit(&self, request: Request<Order>) -> Result<Response<Acceptance>, Status> {
        let order = request.into_inner();
        if (self.capacity as usize) < order.items.len() {
            // Out of capacity
            Ok(Response::new(Acceptance {
                accepted: false,
                reason: String::from("too many items. must be less than 10"),
            }))
        } else {
            let time = order.ordered_time;
            // Build a map from item to unit, and then to history
            let mut history = order
                .items
                .into_iter()
                .fold(HashMap::new(), |mut map, item| {
                    if let Some(record) = map.get_mut(&item) {
                        *record += 1;
                    } else {
                        map.insert(item, 1);
                    }
                    map
                })
                .into_iter()
                .map(|(k, v)| History {
                    time: time.clone(),
                    item: k,
                    unit: v,
                    price: PRICES[k as usize],
                })
                .collect::<Vec<History>>();

            // Update internal table
            let mut tables = self.tables.write().await;
            if let Some(record) = tables.get_mut(&order.table) {
                // Record exists. Append new history.
                record.append(&mut history);
            } else {
                // No record. Insert a new one.
                tables.insert(order.table, history);
            }

            Ok(Response::new(Acceptance {
                accepted: true,
                reason: String::new(),
            }))
        }
    }

    async fn finish(&self, request: Request<Expense>) -> Result<Response<Total>, Status> {
        let expense = request.into_inner();
        let mut tables = self.tables.write().await;
        if let Some(history) = tables.remove(&expense.table) {
            // History exists for the table
            let total = history.iter().map(|h| h.unit as f32 * h.price).sum();
            Ok(Response::new(Total { history, total }))
        } else {
            Err(Status::internal(format!(
                "no order for the table id: {}",
                expense.table
            )))
        }
    }

    async fn cancel(&self, request: Request<Order>) -> Result<Response<()>, Status> {
        // Map from item number to a unit of the item to cancel
        let order = request.into_inner();
        let to_cancel = order
            .items
            .into_iter()
            .fold(HashMap::new(), |mut map, item| {
                if let Some(record) = map.get_mut(&item) {
                    *record += 1;
                } else {
                    map.insert(item, 1);
                }
                map
            });

        if let Some(histories) = self.tables.write().await.get_mut(&order.table) {
            // Table exists
            to_cancel.into_iter().for_each(|(item, mut unit)| {
                *histories = histories
                    .into_iter()
                    .filter_map(|history| {
                        if history.item == item {
                            if history.unit > unit {
                                // Cancel amount is less than the unit of current history.
                                // Subtract by the cancel mount.
                                Some(History {
                                    time: history.time.clone(),
                                    item: history.item,
                                    unit: history.unit - unit,
                                    price: history.price,
                                })
                            } else {
                                // Cancel amount exceeds the unit of the current history.
                                // Reduce the amount to cancel by the unit of the current hostory.
                                unit -= history.unit;
                                // History is removed.
                                None
                            }
                        } else {
                            // The cancelled item is not related to this history.
                            Some(history.clone())
                        }
                    })
                    .collect();
            });
            Ok(Response::new(()))
        } else {
            // There is no history for the table
            Err(Status::internal(format!(
                "no order for the table id: {}",
                order.table
            )))
        }
    }
}

impl RestaurantManager {
    pub(crate) fn new() -> Self {
        Self {
            tables: RwLock::new(HashMap::new()),
            capacity: 9,
        }
    }
}
