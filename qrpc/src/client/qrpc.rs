
use super::ENDPOINT;
use super::proto::example_service::restaurant_client::RestaurantClient;
use super::proto::example_service::{Order, Acceptance, Expense, Total};
use crate::message::{PROTO_FILE_DESCRIPTOR, encode_to_message, decode_message};
use std::sync::RwLock;
use once_cell::sync::Lazy;
use prost_reflect::DynamicMessage;
use tonic::Request;
use tokio::runtime::Builder;
use kdbplus::api::*;

static ERROR_BUFFER: Lazy<RwLock<String>> = Lazy::new(||{
  RwLock::new(String::new())
});

#[no_mangle]
pub extern "C" fn submit_(message: K) -> K{
  let message_descriptor = PROTO_FILE_DESCRIPTOR.get_message_by_name("example_service.Order").unwrap();
  match encode_to_message(message_descriptor, message){
    Ok(dynamic_message) => {      
      let runtime = Builder::new_current_thread().enable_time().enable_io().build().unwrap();
      if let Ok(mut client) = runtime.block_on(RestaurantClient::connect(ENDPOINT.read().expect("failed to get read lock").clone())){
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
            let null_terminated_error = format!("{}\0", error.message());
            buffer.push_str(null_terminated_error.as_str());
            new_error(buffer.as_str())
          }
        }
      }
      else{
        new_error("failed to connect\0")
      }
    },
    Err(error) => new_error(error)
  }
}

#[no_mangle]
pub extern "C" fn finish_(message: K) -> K{
  let message_descriptor = PROTO_FILE_DESCRIPTOR.get_message_by_name("example_service.Expense").unwrap();
  match encode_to_message(message_descriptor, message){
    Ok(dynamic_message) => {      
      let runtime = Builder::new_current_thread().enable_time().enable_io().build().unwrap();
      if let Ok(mut client) = runtime.block_on(RestaurantClient::connect(ENDPOINT.read().expect("failed to get read lock").clone())){
        match runtime.block_on(client.finish(Request::new(dynamic_message.transcode_to::<Expense>().unwrap()))){
          Ok(response) => {
            let message_descriptor = PROTO_FILE_DESCRIPTOR.get_message_by_name("example_service.Total").unwrap();
            let mut dynamic_message = DynamicMessage::new(message_descriptor.clone());
            dynamic_message.transcode_from::<Total>(&response.into_inner()).unwrap();
            decode_message(&dynamic_message, message_descriptor.fields())
          },
          Err(error) => {
            let mut buffer = ERROR_BUFFER.write().expect("failed to get write lock");
            buffer.clear();
            let null_terminated_error = format!("{}\0", error.message());
            buffer.push_str(null_terminated_error.as_str());
            new_error(buffer.as_str())
          }
        }
      }
      else{
        new_error("failed to connect\0")
      }
    },
    Err(error) => new_error(error)
  }
}
