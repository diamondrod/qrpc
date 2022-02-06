


use super::proto::example_service::restaurant_client::RestaurantClient;
use super::proto::example_service::{Order, Acceptance, Expense, Total};
use crate::message::{PROTO_FILE_DESCRIPTOR, encode_to_message, decode_message};
use std::sync::RwLock;
use once_cell::sync::{Lazy, OnceCell};
use prost_reflect::DynamicMessage;
use tonic::{Request, Response, Status};
use tonic::transport::Channel;
use tokio::runtime::Builder;
use kdbplus::api::*;

static ERROR_BUFFER: Lazy<RwLock<String>> = Lazy::new(||{
  RwLock::new(String::new())
});
static GRPC_CLIENT: OnceCell<RestaurantClient<Channel>> = OnceCell::new();

pub extern "C" fn build_client(endpoint: K) -> K{
  match endpoint.get_string(){
    Ok(url) => {
      let runtime = Builder::new_current_thread().build().unwrap();
      match runtime.block_on(RestaurantClient::connect(url)){
        Ok(client) => {
          GRPC_CLIENT.set(client);
          new_symbol("client initialized")
        },
        Err(error) => {
          let mut buffer = ERROR_BUFFER.write().expect("failed to get write lock");
          buffer.clear();
          let null_terminated_error = format!("{}\0", error);
          buffer.push_str(null_terminated_error.as_str());
          new_error(buffer.as_str())
        }
      }
    },
    Err(error) => new_error(error)
  }
}

pub extern "C" fn submit_(message: K) -> K{
  let runtime = Builder::new_current_thread().build().unwrap();
  let message_descriptor = PROTO_FILE_DESCRIPTOR.get_message_by_name("example_service.Order").unwrap();
  match encode_to_message(message_descriptor, message){
    Ok(dynamic_message) => {
      let mut client = GRPC_CLIENT.get().unwrap().clone();
      match runtime.block_on(client.submit(Request::new(dynamic_message.transcode_to::<Order>().unwrap()))){
        Ok(response) => {
          let message_descriptor = PROTO_FILE_DESCRIPTOR.get_message_by_name("example_service.Acceptance").unwrap();
          let mut dynamic_message = DynamicMessage::new(message_descriptor.clone());
          dynamic_message.transcode_from::<Acceptance>(&response.into_inner()).unwrap();
          decode_message(&dynamic_message, message_descriptor.fields())
        },
        Err(error) => {
          let mut buffer = ERROR_BUFFER.write().expect("failed to get write lock");
          buffer.clear();
          let null_terminated_error = format!("{}\0", error);
          buffer.push_str(null_terminated_error.as_str());
          new_error(buffer.as_str())
        }
      }
    },
    Err(error) => new_error(error)
  }
}
