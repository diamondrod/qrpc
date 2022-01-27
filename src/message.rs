//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use std::iter::ExactSizeIterator;
use std::result::Result;
use once_cell::sync::Lazy;
use bytes::Bytes;
use prost::Message;
use prost_reflect::{DynamicMessage, FileDescriptor, Value, ReflectMessage, MessageDescriptor, FieldDescriptor, Kind};
use kdbplus::qtype;
use kdbplus::api::*;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Global Variables
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Bytes representing compiled files.
const PROTO_FILE_DESCRIPTOR_SET_BYTES: &[u8] = include_bytes!("../qrpc_fd_set");
/// File descriptor of compiled files.
static PROTO_FILE_DESCRIPTOR: Lazy<FileDescriptor> = Lazy::new(|| FileDescriptor::decode(PROTO_FILE_DESCRIPTOR_SET_BYTES).unwrap());
/// Symbol message descriptor.
static SYMBOL_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.symbol").unwrap());
/// Timestamp message descriptor.
static TIMESTAMP_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.timestamp").unwrap());
/// Month message descriptor.
static MONTH_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.month").unwrap());
/// Date message descriptor.
static DATE_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.date").unwrap());
/// Datetime message descriptor.
static DATETIME_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.datetime").unwrap());
/// Timespan message descriptor.
static TIMESPAN_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.timespan").unwrap());
/// Minute message descriptor.
static MINUTE_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.minute").unwrap());
/// Second message descriptor.
static SECOND_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.second").unwrap());
/// Time message descriptor.
static TIME_MESSAGE_DESCRIPTOR: Lazy<MessageDescriptor> = Lazy::new(|| PROTO_FILE_DESCRIPTOR.get_message_by_name("q.time").unwrap());

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Encode q dictionary to protobuf encoded bytes.
/// # Parameters
/// - `message_type`: Message type with package name prefix, e.g., `example.Scalar`.
/// - `data`: q dictionary.
#[no_mangle]
pub extern "C" fn encode(message: K, data: K) -> K{
  match message.get_symbol(){
    Ok(message_name) => {
      if let Some(message_descriptor) = PROTO_FILE_DESCRIPTOR.get_message_by_name(message_name){
        match encode_to_message(message_descriptor, data){
          Ok(dynamic_message) => {
            let encoded = dynamic_message.encode_to_vec();
            let bytes = new_list(qtype::BYTE_LIST, encoded.len() as i64);
            bytes.as_mut_slice::<G>().copy_from_slice(&encoded);
            bytes 
          },
          Err(error) => new_error(error)
        }
      }
      else{
        // Specified message type was not found
        new_error("no such message type\0")
      }
    },
    Err(error) => new_error(error)
  }  
}

/// Decode protobuf encoded bytes to q dictionary.
/// # Parameters
/// - `message_type`: Message type with package name prefix, e.g., `example.Scalar`.
/// - `bytes`: Protobuf encoded bytes.
#[no_mangle]
pub extern "C" fn decode(message: K, bytes: K) -> K{
  match message.get_symbol(){
    Ok(message_name) => {
      if let Some(message_descriptor) = PROTO_FILE_DESCRIPTOR.get_message_by_name(message_name){
        let fields = message_descriptor.fields();
        if let Ok(dynamic_message) = DynamicMessage::decode(message_descriptor.clone(), &*bytes.as_mut_slice::<G>()){
          decode_message(&dynamic_message, fields)
        }
        else{
          new_error("failed to decode message\0")
        }
      }
      else{
        // Specified message type was not found
        new_error("no such message type\0")
      }
    },
    Err(error) => new_error(error)
  }  
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Private Functions
//++++++++++++++++++++++++++++++++++++++++++++++++++//

fn set_bool_to_message(value: bool, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Bool => {
      dynamic_message.set_field(field, Value::Bool(value));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_int_to_message(value: i32, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Int32 | Kind::Sint32 => {
      dynamic_message.set_field(field, Value::I32(value));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_long_to_message(value: i64, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Int64 | Kind::Sint64 => {
      dynamic_message.set_field(field, Value::I64(value));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_real_to_message(value: f32, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Float => {
      dynamic_message.set_field(field, Value::F32(value));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_float_to_message(value: f64, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Double => {
      dynamic_message.set_field(field, Value::F64(value));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_symbol_to_message(value: &str, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.symbol" => {
      let mut inner = DynamicMessage::new(SYMBOL_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("symbol", Value::String(value.to_string()));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_timestamp_to_message(value: i64, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.timestamp" => {
      let mut inner = DynamicMessage::new(TIMESTAMP_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("nanos", Value::I64(value));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_month_to_message(value: i32, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.month" => {
      let mut inner = DynamicMessage::new(MONTH_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("months", Value::I32(value));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_date_to_message(value: i32, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.date" => {
      let mut inner = DynamicMessage::new(DATE_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("days", Value::I32(value));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_datetime_to_message(value: f64, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.datetime" => {
      let mut inner = DynamicMessage::new(DATETIME_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("days", Value::F64(value));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_timespan_to_message(value: i64, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.timespan" => {
      let mut inner = DynamicMessage::new(TIMESPAN_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("nanos", Value::I64(value));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_minute_to_message(value: i32, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.minute" => {
      let mut inner = DynamicMessage::new(MINUTE_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("minutes", Value::I32(value));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_second_to_message(value: i32, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.second" => {
      let mut inner = DynamicMessage::new(SECOND_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("seconds", Value::I32(value));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_time_to_message(value: i32, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message) if message.full_name() == "q.time" => {
      let mut inner = DynamicMessage::new(TIME_MESSAGE_DESCRIPTOR.clone());
      inner.set_field_by_name("millis", Value::I32(value));
      dynamic_message.set_field(&field, Value::Message(inner));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_string_to_message(value: String, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::String => {
      dynamic_message.set_field(field, Value::String(value));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_bytes_to_message(value: &[u8], field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::String => {
      dynamic_message.set_field(field, Value::Bytes(Bytes::copy_from_slice(value)));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

fn set_dictionary_to_message(value: K, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match field.kind(){
    Kind::Message(message_descriptor) => {
      let encoded = encode_to_message(message_descriptor, value)?;
      dynamic_message.set_field(field, Value::Message(encoded));
      Ok(())
    },
    _ => Err("type mismatch\0")
  }
}

/// Check if q object type matches a field type.
fn set_value_to_message(value: K, field: &FieldDescriptor, dynamic_message: &mut DynamicMessage) -> Result<(), &'static str>{
  match value.get_type(){
    qtype::BOOL_ATOM => set_bool_to_message(value.get_bool().unwrap(), field, dynamic_message),
    qtype::INT_ATOM => set_int_to_message(value.get_int().unwrap(), field, dynamic_message),
    qtype::LONG_ATOM => set_long_to_message(value.get_long().unwrap(), field, dynamic_message),
    qtype::REAL_ATOM => set_real_to_message(value.get_real().unwrap(), field, dynamic_message),
    qtype::FLOAT_ATOM => set_float_to_message(value.get_float().unwrap(), field, dynamic_message),
    qtype::STRING => set_string_to_message(value.get_string().unwrap(), field, dynamic_message),
    qtype::BYTE_LIST => set_bytes_to_message(value.as_mut_slice::<G>(), field, dynamic_message),
    qtype::SYMBOL_ATOM => set_symbol_to_message(value.get_symbol().unwrap(), field, dynamic_message),
    qtype::TIMESTAMP_ATOM => set_timestamp_to_message(value.get_long().unwrap(), field, dynamic_message),
    qtype::MONTH_ATOM => set_month_to_message(value.get_int().unwrap(), field, dynamic_message),
    qtype::DATE_ATOM => set_date_to_message(value.get_int().unwrap(), field, dynamic_message),
    qtype::DATETIME_ATOM => set_datetime_to_message(value.get_float().unwrap(), field, dynamic_message),
    qtype::TIMESPAN_ATOM => set_timespan_to_message(value.get_long().unwrap(), field, dynamic_message),
    qtype::MINUTE_ATOM => set_minute_to_message(value.get_int().unwrap(), field, dynamic_message),
    qtype::SECOND_ATOM => set_second_to_message(value.get_int().unwrap(), field, dynamic_message),
    qtype::TIME_ATOM => set_time_to_message(value.get_int().unwrap(), field, dynamic_message),
    qtype::DICTIONARY => set_dictionary_to_message(value, field, dynamic_message),
    _ => Err("unsupported type\0")
  }
}

/// Encode q dictionary to dynamic message.
fn encode_to_message(message_descriptor: MessageDescriptor, data: K) -> Result<DynamicMessage, &'static str>{
  let mut dynamic_message = DynamicMessage::new(message_descriptor);
  let keys = data.as_mut_slice::<K>()[0].as_mut_slice::<S>();
  let values_ = data.as_mut_slice::<K>()[1];
  match values_.get_type(){
    qtype::BOOL_LIST => {
      let values = values_.as_mut_slice::<G>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_bool_to_message(*value != 0, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::INT_LIST => {
      let values = values_.as_mut_slice::<I>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_int_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::LONG_LIST => {
      let values = values_.as_mut_slice::<J>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_long_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::REAL_LIST => {
      let values = values_.as_mut_slice::<E>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_real_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::FLOAT_LIST => {
      let values = values_.as_mut_slice::<F>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_float_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::SYMBOL_LIST => {
      let values = values_.as_mut_slice::<S>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_symbol_to_message(S_to_str(*value), &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::TIMESTAMP_LIST => {
      let values = values_.as_mut_slice::<J>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_timestamp_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::MONTH_LIST => {
      let values = values_.as_mut_slice::<I>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_month_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::DATE_LIST => {
      let values = values_.as_mut_slice::<I>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_date_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::DATETIME_LIST => {
      let values = values_.as_mut_slice::<F>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_datetime_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::TIMESPAN_LIST => {
      let values = values_.as_mut_slice::<J>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_timespan_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::MINUTE_LIST => {
      let values = values_.as_mut_slice::<I>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_minute_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::SECOND_LIST => {
      let values = values_.as_mut_slice::<I>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_second_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::TIME_LIST => {
      let values = values_.as_mut_slice::<I>();
      for (key, value) in keys.iter().zip(values){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          set_time_to_message(*value, &field, &mut dynamic_message)?;
        }             
      }
    },
    qtype::COMPOUND_LIST => {
      let values = values_.as_mut_slice::<K>();
      for i in 0 .. keys.len(){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(keys[i])){
          set_value_to_message(values[i], &field, &mut dynamic_message)?;
        }
      }
    },
    _ => unimplemented!()
  }
  Ok(dynamic_message)
}

/// Convert dynamic message into q dictionary.
fn decode_message(dynamic_message: &DynamicMessage, fields: impl ExactSizeIterator::<Item = FieldDescriptor>) -> K{
  let keys = new_list(qtype::SYMBOL_LIST, fields.len() as i64);
  let keys_slice = keys.as_mut_slice::<S>();
  let mut simple = KNULL;
  let mut compound = KNULL;
  let mut list_type = qtype::NULL;
  let mut i = 0;
  fields.for_each(|field|{
    // Store field name as a key
    keys_slice[i] = enumerate(str_to_S!(field.name()));
    // Decode value
    if let Some(v_) = dynamic_message.get_field_by_name(field.name()){
      match v_.as_ref(){
        Value::Bool(v) => {
          // Bool
          match list_type{
            qtype::NULL =>{
              list_type = qtype::BOOL_LIST;
              simple = new_list(qtype::BOOL_LIST, 0);
              simple.push_raw(*v).unwrap();
            },
            qtype::BOOL_LIST => {
              simple.push_raw(*v).unwrap();
            },
            qtype::COMPOUND_LIST => {
              compound.push(new_bool(*v as i32)).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_bool(*v as i32)).unwrap();
            }
          }
        },
        Value::I32(v) => {
          // Int
          match list_type{
            qtype::NULL =>{
              list_type = qtype::INT_LIST;
              simple = new_list(qtype::INT_LIST, 0);
              simple.push_raw(*v).unwrap();
            },
            qtype::INT_LIST => {
              simple.push_raw(*v).unwrap();
            },
            qtype::COMPOUND_LIST => {
              compound.push(new_int(*v)).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_int(*v)).unwrap();
            }
          }
        },
        Value::I64(v) => {
          // Long
          match list_type{
            qtype::NULL =>{
              list_type = qtype::LONG_LIST;
              simple = new_list(qtype::LONG_LIST, 0);
              simple.push_raw(*v).unwrap();
            },
            qtype::LONG_LIST => {
              simple.push_raw(*v).unwrap();
            },
            qtype::COMPOUND_LIST => {
              compound.push(new_long(*v)).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_long(*v)).unwrap();
            }
          }
        },
        Value::F32(v) => {
          // Real
          match list_type{
            qtype::NULL =>{
              list_type = qtype::REAL_LIST;
              simple = new_list(qtype::REAL_LIST, 0);
              simple.push_raw(*v as f64).unwrap();
            },
            qtype::REAL_LIST => {
              simple.push_raw(*v as f64).unwrap();
            },
            qtype::COMPOUND_LIST => {
              compound.push(new_real(*v as f64)).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_real(*v as f64)).unwrap();
            }
          }
        },
        Value::F64(v) => {
          // Float
          match list_type{
            qtype::NULL =>{
              list_type = qtype::FLOAT_LIST;
              simple = new_list(qtype::FLOAT_LIST, 0);
              simple.push_raw(*v).unwrap();
            },
            qtype::FLOAT_LIST => {
              simple.push_raw(*v).unwrap();
            },
            qtype::COMPOUND_LIST => {
              compound.push(new_float(*v)).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_float(*v)).unwrap();
            }
          }
        },
        Value::String(v) => {
          // String
          match list_type{
            qtype::NULL =>{
              list_type = qtype::COMPOUND_LIST;
              compound = new_list(qtype::COMPOUND_LIST, 0);
              compound.push(new_string(v)).unwrap();
            },
            qtype::COMPOUND_LIST => {
              compound.push(new_string(v)).unwrap();
            },
            _ => {
              // Simple list or null
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_string(v)).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.symbol" => {
          // Symbol
          let v = message.get_field_by_name("symbol").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::SYMBOL_LIST;
              simple = new_list(qtype::SYMBOL_LIST, 0);
              simple.push_symbol(v.as_str().unwrap()).unwrap();
            },
            qtype::SYMBOL_LIST => {
              simple.push_symbol(v.as_str().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_symbol(v.as_str().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_symbol(v.as_str().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.timestamp" => {
          // Timestamp
          let v = message.get_field_by_name("nanos").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::TIMESTAMP_LIST;
              simple = new_list(qtype::TIMESTAMP_LIST, 0);
              simple.push_raw(v.as_i64().unwrap()).unwrap();
            },
            qtype::TIMESTAMP_LIST => {
              simple.push_raw(v.as_i64().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_timestamp(v.as_i64().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_timestamp(v.as_i64().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.month" => {
          // Month
          let v = message.get_field_by_name("months").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::MONTH_LIST;
              simple = new_list(qtype::MONTH_LIST, 0);
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            },
            qtype::MONTH_LIST => {
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_month(v.as_i32().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_month(v.as_i32().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.date" => {
          // Date
          let v = message.get_field_by_name("days").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::DATE_LIST;
              simple = new_list(qtype::DATE_LIST, 0);
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            },
            qtype::DATE_LIST => {
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_date(v.as_i32().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_date(v.as_i32().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.datetime" => {
          // Datetime
          let v = message.get_field_by_name("days").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::DATETIME_LIST;
              simple = new_list(qtype::DATETIME_LIST, 0);
              simple.push_raw(v.as_f64().unwrap()).unwrap();
            },
            qtype::DATETIME_LIST => {
              simple.push_raw(v.as_f64().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_datetime(v.as_f64().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_datetime(v.as_f64().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.timespan" => {
          // Timespan
          let v = message.get_field_by_name("nanos").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::TIMESPAN_LIST;
              simple = new_list(qtype::TIMESPAN_LIST, 0);
              simple.push_raw(v.as_i64().unwrap()).unwrap();
            },
            qtype::TIMESPAN_LIST => {
              simple.push_raw(v.as_i64().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_timespan(v.as_i64().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_timespan(v.as_i64().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.minute" => {
          // Minute
          let v = message.get_field_by_name("minutes").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::MINUTE_LIST;
              simple = new_list(qtype::MINUTE_LIST, 0);
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            },
            qtype::MINUTE_LIST => {
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_minute(v.as_i32().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_minute(v.as_i32().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.second" => {
          // Second
          let v = message.get_field_by_name("seconds").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::SECOND_LIST;
              simple = new_list(qtype::SECOND_LIST, 0);
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            },
            qtype::SECOND_LIST => {
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_second(v.as_i32().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_second(v.as_i32().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) if message.descriptor().full_name() == "q.time" => {
          // Time
          let v = message.get_field_by_name("millis").unwrap();
          match list_type{
            qtype::NULL =>{
              list_type = qtype::TIME_LIST;
              simple = new_list(qtype::TIME_LIST, 0);
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            },
            qtype::TIME_LIST => {
              simple.push_raw(v.as_i32().unwrap()).unwrap();
            }
            qtype::COMPOUND_LIST => {
              compound.push(new_time(v.as_i32().unwrap())).unwrap();
            },
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
              compound.push(new_time(v.as_i32().unwrap())).unwrap();
            }
          }
        },
        Value::Message(message) => {
          // Protobuf message
          let message_descriptor = message.descriptor();
          let inner_fields =message_descriptor.fields();
          let v = decode_message(message, inner_fields);
          // Move to compound list
          list_type = qtype::COMPOUND_LIST;
          match list_type{
            qtype::NULL =>{
              compound = new_list(qtype::COMPOUND_LIST, 0);
              compound.push(v).unwrap();
            },
            _ => {
              compound = simple_to_compound(simple);
              compound.push(v).unwrap();
            }
          }
        },
        _ => unimplemented!()
      }
    }
    else{
      // Move to compound list
      list_type = qtype::COMPOUND_LIST;
      match list_type{
        qtype::NULL =>{
          compound = new_list(qtype::COMPOUND_LIST, 0);
          compound.push(new_null()).unwrap();
        },
        _ => {
          compound = simple_to_compound(simple);
          compound.push(new_null()).unwrap();
        }
      }
    }
    i += 1;
  });
  match list_type{
    qtype::COMPOUND_LIST => new_dictionary(keys, compound),
    _ => new_dictionary(keys, simple)
  }
}
