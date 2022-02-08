//! This odule provides gRPC client implementation.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

mod proto;
mod rpc;

use std::sync::RwLock;
use kdbplus::api::*;
use once_cell::sync::Lazy;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Global Variables
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Endpoint to connect at a call of gRPC service.
static ENDPOINT: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::new()));

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Interface
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Set endpoint to connect.
#[no_mangle]
pub extern "C" fn set_endpoint(endpoint: K) -> K{
  match endpoint.get_string(){
    Ok(url) => {
      {
        let mut endpoint = ENDPOINT.write().expect("failed to get write lock");
        endpoint.clear();
        endpoint.push_str(&url);
      }
      new_string("endpoint was set")
    },
    Err(error) => new_error(error)
  }
}
