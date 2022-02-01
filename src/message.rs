//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use std::collections::HashMap;
use std::iter::ExactSizeIterator;
use std::result::Result;
use once_cell::sync::Lazy;
use bytes::Bytes;
use prost::Message;
use prost_reflect::{DynamicMessage, FileDescriptor, Value, ReflectMessage, MessageDescriptor, FieldDescriptor, Kind, MapKey};
use kdbplus::qtype;
use kdbplus::api::*;
use kdbplus::api::native::k;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Global Variables
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Bytes representing compiled files.
const PROTO_FILE_DESCRIPTOR_SET_BYTES: &[u8] = include_bytes!("../qrpc_fd_set");
/// File descriptor of compiled files.
static PROTO_FILE_DESCRIPTOR: Lazy<FileDescriptor> = Lazy::new(|| FileDescriptor::decode(PROTO_FILE_DESCRIPTOR_SET_BYTES).unwrap());

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

//%% Encode %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Convert underlying int value to `Value`.
fn int_to_value(value: i32, field: &FieldDescriptor) -> Result<Value, &'static str>{
  match field.kind(){
    // Int
    Kind::Int32 | Kind::Sint32 => Ok(Value::I32(value)),
    // Month
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.month" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("months", Value::I32(value));
      Ok(Value::Message(inner))
    },
    // Date
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.date" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("days", Value::I32(value));
      Ok(Value::Message(inner))
    },
    // Minute
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.minute" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("minutes", Value::I32(value));
      Ok(Value::Message(inner))
    },
    // Second
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.second" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("seconds", Value::I32(value));
      Ok(Value::Message(inner))
    },
    // Time
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.time" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("millis", Value::I32(value));
      Ok(Value::Message(inner))
    },
    // There are no other int compatible type
    _ => Err("non-int value\0")
  }
}

/// Convert underlying long value to `Value`.
fn long_to_value(value: i64, field: &FieldDescriptor) -> Result<Value, &'static str>{
  match field.kind(){
    // Long
    Kind::Int64 | Kind::Sint64 => Ok(Value::I64(value)),
    // Timestamp
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.timestamp" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("nanos", Value::I64(value));
      Ok(Value::Message(inner))
    },
    // Timespan
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.timespan" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("nanos", Value::I64(value));
      Ok(Value::Message(inner))
    },
    // There are no other long compatible type
    _ => Err("non-long value\0")
  }
}

/// Convert underlying float value to `Value`.
fn float_to_value(value: f64, field: &FieldDescriptor) -> Result<Value, &'static str>{
  match field.kind(){
    // Float
    Kind::Double => Ok(Value::F64(value)),
    // Datetime
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.datetime" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("days", Value::F64(value));
      Ok(Value::Message(inner))
    },
    // There are no other float compatible type
    _ => Err("non-float value\0")
  }
}

/// Convert underlying symbol value to `Value`.
fn symbol_to_value(value: S, field: &FieldDescriptor) -> Result<Value, &'static str>{
  match field.kind(){
    // Datetime
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.symbol" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("symbol", Value::String(S_to_str(value).to_string()));
      Ok(Value::Message(inner))
    },
    // There are no other float compatible type
    _ => Err("non-symbol value\0")
  }
}

/// Convert q dictionary to protobuf map type specified by a given message descriptor.
fn k_to_map(value: K, message_descriptor: &MessageDescriptor) -> Result<Value, &'static str>{
  // Map field equivalent of repeated map entry composed of `key = 1` and `value = 2`.
  let keys = value.as_mut_slice::<K>()[0];
  let values = value.as_mut_slice::<K>()[1];
  let value_field_descriptor = message_descriptor.map_entry_value_field();
  let mut map = HashMap::new();
  // Match field kind and q value type
  match message_descriptor.map_entry_key_field().kind(){
    Kind::Bool => {
      if keys.get_type() == qtype::BOOL_LIST{
        match values.get_type(){
          qtype::BOOL_LIST => {
            keys.as_mut_slice::<G>().iter().zip(values.as_mut_slice::<G>()).for_each(|(key, value)|{
              map.insert(MapKey::Bool(*key != 0), Value::Bool(*value != 0));
            });
          },
          qtype::INT_LIST | qtype::MONTH_LIST | qtype::DATE_LIST | qtype::MINUTE_LIST | qtype::SECOND_LIST | qtype::TIME_LIST => {
            keys.as_mut_slice::<G>().iter().zip(values.as_mut_slice::<I>()).map(|(key, value)|{
              map.insert(MapKey::Bool(*key != 0), int_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::LONG_LIST | qtype::TIMESTAMP_LIST | qtype::TIMESPAN_LIST => {
            keys.as_mut_slice::<G>().iter().zip(values.as_mut_slice::<J>()).map(|(key, value)|{
              map.insert(MapKey::Bool(*key != 0), long_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::REAL_LIST => {
            keys.as_mut_slice::<G>().iter().zip(values.as_mut_slice::<E>()).for_each(|(key, value)|{
              map.insert(MapKey::Bool(*key != 0), Value::F32(*value));
            });
          },
          qtype::FLOAT_LIST | qtype::DATETIME_LIST => {
            keys.as_mut_slice::<G>().iter().zip(values.as_mut_slice::<F>()).map(|(key, value)|{
              map.insert(MapKey::Bool(*key != 0), float_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::SYMBOL_LIST => {
            keys.as_mut_slice::<G>().iter().zip(values.as_mut_slice::<S>()).map(|(key, value)|{
              map.insert(MapKey::Bool(*key != 0), symbol_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::TABLE => {
            // q converts list of dictionaries into table
            // Hence table must be treated by getting each row.
            keys.as_mut_slice::<G>().iter().enumerate().map(|(i, key)|{
              let row = values.get_row(i).unwrap();
              map.insert(MapKey::Bool(*key != 0), k_to_value(row, &value_field_descriptor)?);
              decrement_reference_count(row);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          _ => {
            keys.as_mut_slice::<G>().iter().zip(values.as_mut_slice::<K>()).map(|(key, value)|{
              map.insert(MapKey::Bool(*key != 0), k_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          }
        }
      }
      else{
        return Err("type mismatch. expected: bool list\0")
      }
    },
    Kind::Int32 | Kind::Sint32 => {
      if keys.get_type() == qtype::INT_LIST{
        match values.get_type(){
          qtype::BOOL_LIST => {
            keys.as_mut_slice::<I>().iter().zip(values.as_mut_slice::<G>()).for_each(|(key, value)|{
              map.insert(MapKey::I32(*key), Value::Bool(*value != 0));
            });
          },
          qtype::INT_LIST | qtype::MONTH_LIST | qtype::DATE_LIST | qtype::MINUTE_LIST | qtype::SECOND_LIST | qtype::TIME_LIST => {
            keys.as_mut_slice::<I>().iter().zip(values.as_mut_slice::<I>()).map(|(key, value)|{
              map.insert(MapKey::I32(*key), int_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::LONG_LIST | qtype::TIMESTAMP_LIST | qtype::TIMESPAN_LIST => {
            keys.as_mut_slice::<I>().iter().zip(values.as_mut_slice::<J>()).map(|(key, value)|{
              map.insert(MapKey::I32(*key), long_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::REAL_LIST => {
            keys.as_mut_slice::<I>().iter().zip(values.as_mut_slice::<E>()).for_each(|(key, value)|{
              map.insert(MapKey::I32(*key), Value::F32(*value));
            });
          },
          qtype::FLOAT_LIST | qtype::DATETIME_LIST => {
            keys.as_mut_slice::<I>().iter().zip(values.as_mut_slice::<F>()).map(|(key, value)|{
              map.insert(MapKey::I32(*key), float_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::SYMBOL_LIST => {
            keys.as_mut_slice::<I>().iter().zip(values.as_mut_slice::<S>()).map(|(key, value)|{
              map.insert(MapKey::I32(*key), symbol_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::TABLE => {
            // q converts list of dictionaries into table
            // Hence table must be treated by getting each row.
            keys.as_mut_slice::<I>().iter().enumerate().map(|(i, key)|{
              let row = values.get_row(i).unwrap();
              map.insert(MapKey::I32(*key), k_to_value(row, &value_field_descriptor)?);
              decrement_reference_count(row);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          _ => {
            keys.as_mut_slice::<I>().iter().zip(values.as_mut_slice::<K>()).map(|(key, value)|{
              map.insert(MapKey::I32(*key), k_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          }
        }
      }
      else{
        return Err("type mismatch. expected: int list\0")
      }
    },
    Kind::Int64 | Kind::Sint64 => {
      if keys.get_type() == qtype::LONG_LIST{
        match values.get_type(){
          qtype::BOOL_LIST => {
            keys.as_mut_slice::<J>().iter().zip(values.as_mut_slice::<G>()).for_each(|(key, value)|{
              map.insert(MapKey::I64(*key), Value::Bool(*value != 0));
            });
          },
          qtype::INT_LIST | qtype::MONTH_LIST | qtype::DATE_LIST | qtype::MINUTE_LIST | qtype::SECOND_LIST | qtype::TIME_LIST => {
            keys.as_mut_slice::<J>().iter().zip(values.as_mut_slice::<I>()).map(|(key, value)|{
              map.insert(MapKey::I64(*key), int_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::LONG_LIST | qtype::TIMESTAMP_LIST | qtype::TIMESPAN_LIST => {
            keys.as_mut_slice::<J>().iter().zip(values.as_mut_slice::<J>()).map(|(key, value)|{
              map.insert(MapKey::I64(*key), long_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::REAL_LIST => {
            keys.as_mut_slice::<J>().iter().zip(values.as_mut_slice::<E>()).for_each(|(key, value)|{
              map.insert(MapKey::I64(*key), Value::F32(*value));
            });
          },
          qtype::FLOAT_LIST | qtype::DATETIME_LIST => {
            keys.as_mut_slice::<J>().iter().zip(values.as_mut_slice::<F>()).map(|(key, value)|{
              map.insert(MapKey::I64(*key), float_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::SYMBOL_LIST => {
            keys.as_mut_slice::<J>().iter().zip(values.as_mut_slice::<S>()).map(|(key, value)|{
              map.insert(MapKey::I64(*key), symbol_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::TABLE => {
            // q converts list of dictionaries into table
            // Hence table must be treated by getting each row.
            keys.as_mut_slice::<J>().iter().enumerate().map(|(i, key)|{
              let row = values.get_row(i).unwrap();
              map.insert(MapKey::I64(*key), k_to_value(row, &value_field_descriptor)?);
              decrement_reference_count(row);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          _ => {
            keys.as_mut_slice::<J>().iter().zip(values.as_mut_slice::<K>()).map(|(key, value)|{
              map.insert(MapKey::I64(*key), k_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          }
        }
      }
      else{
        return Err("type mismatch. expected: long list\0")
      }
    },
    Kind::String => {
      // Interpret string key as symbol list
      if keys.get_type() == qtype::SYMBOL_LIST{
        match values.get_type(){
          qtype::BOOL_LIST => {
            keys.as_mut_slice::<S>().iter().zip(values.as_mut_slice::<G>()).for_each(|(key, value)|{
              map.insert(MapKey::String(S_to_str(*key).to_string()), Value::Bool(*value != 0));
            });
          },
          qtype::INT_LIST | qtype::MONTH_LIST | qtype::DATE_LIST | qtype::MINUTE_LIST | qtype::SECOND_LIST | qtype::TIME_LIST => {
            keys.as_mut_slice::<S>().iter().zip(values.as_mut_slice::<I>()).map(|(key, value)|{
              map.insert(MapKey::String(S_to_str(*key).to_string()), int_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::LONG_LIST | qtype::TIMESTAMP_LIST | qtype::TIMESPAN_LIST => {
            keys.as_mut_slice::<S>().iter().zip(values.as_mut_slice::<J>()).map(|(key, value)|{
              map.insert(MapKey::String(S_to_str(*key).to_string()), long_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::REAL_LIST => {
            keys.as_mut_slice::<S>().iter().zip(values.as_mut_slice::<E>()).for_each(|(key, value)|{
              map.insert(MapKey::String(S_to_str(*key).to_string()), Value::F32(*value));
            });
          },
          qtype::FLOAT_LIST | qtype::DATETIME_LIST => {
            keys.as_mut_slice::<S>().iter().zip(values.as_mut_slice::<F>()).map(|(key, value)|{
              map.insert(MapKey::String(S_to_str(*key).to_string()), float_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::SYMBOL_LIST => {
            keys.as_mut_slice::<I>().iter().zip(values.as_mut_slice::<S>()).map(|(key, value)|{
              map.insert(MapKey::I32(*key), symbol_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          qtype::TABLE => {
            // q converts list of dictionaries into table
            // Hence table must be treated by getting each row.
            keys.as_mut_slice::<S>().iter().enumerate().map(|(i, key)|{
              let row = values.get_row(i).unwrap();
              map.insert(MapKey::String(S_to_str(*key).to_string()), k_to_value(row, &value_field_descriptor)?);
              decrement_reference_count(row);
              Ok(())
            }).collect::<Result<(), _>>()?;
          },
          _ => {
            keys.as_mut_slice::<S>().iter().zip(values.as_mut_slice::<K>()).map(|(key, value)|{
              map.insert(MapKey::String(S_to_str(*key).to_string()), k_to_value(*value, &value_field_descriptor)?);
              Ok(())
            }).collect::<Result<(), _>>()?;
          }
        }
      }
      else{
        return Err("type mismatch. expected: symbol list\0")
      }
    },
    _ => return Err("unsipported key type\0")
  }
  Ok(Value::Map(map))
}

/// Convert q object to `Value` specified by a given field descriptor.
fn k_to_value(value: K, field: &FieldDescriptor) -> Result<Value, &'static str>{
  match field.kind(){
    // Repeated bool
    Kind::Bool if field.is_list() => {
      if value.get_type() == qtype::BOOL_LIST{
        Ok(Value::List(value.as_mut_slice::<G>().iter().map(|b| Value::Bool(*b != 0)).collect()))
      }
      else{
        Err("type mismatch. expected: bool list\0")
      }
    },
    // Bytes
    Kind::Bytes => {
      if value.get_type() == qtype::BYTE_LIST{
        Ok(Value::Bytes(Bytes::copy_from_slice(value.as_mut_slice::<G>())))
      }
      else{
        Err("type mismatch. expected: byte list\0")
      }
    },
    // Repeated int
    Kind::Int32 | Kind::Sint32 if field.is_list() => {
      if value.get_type() == qtype::INT_LIST{
        Ok(Value::List(value.as_mut_slice::<I>().iter().map(|int| Value::I32(*int)).collect()))
      }
      else{
        Err("type mismatch. expected: int list\0")
      }
    },
    // Repeated long
    Kind::Int64 | Kind::Sint64 if field.is_list() => {
      if value.get_type() == qtype::LONG_LIST{
        Ok(Value::List(value.as_mut_slice::<J>().iter().map(|long| Value::I64(*long)).collect()))
      }
      else{
        Err("type mismatch. expected: long list\0")
      }
    },
    // Repeated real
    Kind::Float if field.is_list() => {
      if value.get_type() == qtype::REAL_LIST{
        Ok(Value::List(value.as_mut_slice::<E>().iter().map(|real| Value::F32(*real)).collect()))
      }
      else{
        Err("type mismatch. expected: real list\0")
      }
    },
    // Repeated float
    Kind::Double if field.is_list() => {
      if value.get_type() == qtype::FLOAT_LIST{
        Ok(Value::List(value.as_mut_slice::<F>().iter().map(|float| Value::F64(*float)).collect()))
      }
      else{
        Err("type mismatch. expected: float list\0")
      }
    },
    // String
    Kind::String => Ok(Value::String(value.get_string()?)),
    // Repeated symbol
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.symbol" => {
      if value.get_type() == qtype::SYMBOL_LIST{
        Ok(Value::List(value.as_mut_slice::<S>().iter().map(|symbol|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("symbol", Value::String(S_to_str(*symbol).to_string()));
          Value::Message(inner)
        }).collect()))
      }
      else{
        Err("type mismatch. expected: symbol list\0")
      }
    },
    // Repeated timestamp
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.timestamp" => {
      if value.get_type() == qtype::TIMESTAMP_LIST{
        Ok(Value::List(value.as_mut_slice::<J>().iter().map(|value|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("nanos", Value::I64(*value));
          Value::Message(inner)
        }).collect()))
      }
      else{
        Err("type mismatch. expected: timestamp list\0")
      }
    },
    // Repeated month
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.month" => {
      if value.get_type() == qtype::MONTH_LIST{
        Ok(Value::List(value.as_mut_slice::<I>().iter().map(|value|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("months", Value::I32(*value));
          Value::Message(inner)
        }).collect()))
      }
      else{
        Err("type mismatch. expected: month list\0")
      }
    },
    // Repeated date
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.date" => {
      if value.get_type() == qtype::DATE_LIST{
        Ok(Value::List(value.as_mut_slice::<I>().iter().map(|value|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("days", Value::I32(*value));
          Value::Message(inner)
        }).collect()))
      }
       else{
        Err("type mismatch. expected: date list\0")
       } 
    },
    // Repeated datetime
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.datetime" => {
      if value.get_type() == qtype::DATETIME_LIST{
        Ok(Value::List(value.as_mut_slice::<F>().iter().map(|value|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("days", Value::F64(*value));
          Value::Message(inner)
        }).collect()))
      }
      else{
        Err("type mismatch. expected: datetime list\0")
      }
    },
    // Repeated timespan
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.timespan" => {
      if value.get_type() == qtype::TIMESPAN_LIST{
        Ok(Value::List(value.as_mut_slice::<J>().iter().map(|value|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("nanos", Value::I64(*value));
          Value::Message(inner)
        }).collect()))
      }
      else{
        Err("type mismatch. expected: timespan list\0")
      }
    },
    // Repeated minute
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.minute" => {
      if value.get_type() == qtype::MINUTE_LIST{
        Ok(Value::List(value.as_mut_slice::<I>().iter().map(|value|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("minutes", Value::I32(*value));
          Value::Message(inner)
        }).collect()))
      }
      else{
        Err("type mismatch. expected: minute list\0")
      }
    },
    // Repeated second
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.second" => {
      if value.get_type() == qtype::SECOND_LIST{
        Ok(Value::List(value.as_mut_slice::<I>().iter().map(|value|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("seconds", Value::I32(*value));
          Value::Message(inner)
        }).collect()))
      }
      else{
        Err("type mismatch. expected: second list\0")
      }
    },
    // Repeated time
    Kind::Message(message_descriptor) if field.is_list() && message_descriptor.full_name() == "q.time" => {
      if value.get_type() == qtype::TIME_LIST{
        Ok(Value::List(value.as_mut_slice::<I>().iter().map(|value|{
          let mut inner = DynamicMessage::new(message_descriptor.clone());
          inner.set_field_by_name("millis", Value::I32(*value));
          Value::Message(inner)
        }).collect()))
      }
      else{
        Err("type mismatch. expected: time list\0")
      }
    },
    // Repeated protobuf message
    Kind::Message(message_descriptor) if field.is_list() => {
      if value.get_type() == qtype::TABLE{
        Ok(Value::List((0..value.len() as usize).into_iter().map(|i|{
          let row = value.get_row(i).unwrap();
          let encoded = Value::Message(encode_to_message(message_descriptor.clone(), row)?);
          decrement_reference_count(row);
          Ok(encoded)
        }).collect::<Result<Vec<Value>, &'static str>>()?))
      }
      else{
        Err("type mismatch. expected: table\0")
      }
    },
    // Bool
    Kind::Bool => Ok(Value::Bool(value.get_bool()?)),
    // Int
    Kind::Int32 | Kind::Sint32 => Ok(Value::I32(value.get_int()?)),
    // Long
    Kind::Int64 | Kind::Sint64 => Ok(Value::I64(value.get_long()?)),
    // Real
    Kind::Float => Ok(Value::F32(value.get_real()?)),
    // Float
    Kind::Double => Ok(Value::F64(value.get_float()?)),
    // Symbol
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.symbol" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("symbol", Value::String(value.get_symbol()?.to_string()));
      Ok(Value::Message(inner))
    },
    // Timestamp
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.timestamp" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("nanos", Value::I64(value.get_long()?));
      Ok(Value::Message(inner))
    },
    // Month
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.month" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("months", Value::I32(value.get_int()?));
      Ok(Value::Message(inner))
    },
    // Date
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.date" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("days", Value::I32(value.get_int()?));
      Ok(Value::Message(inner))
    },
    // Datetime
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.datetime" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("days", Value::F64(value.get_float()?));
      Ok(Value::Message(inner))
    },
    // Timespan
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.timespan" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("nanos", Value::I64(value.get_long()?));
      Ok(Value::Message(inner))
    },
    // Minute
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.minute" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("minutes", Value::I32(value.get_int()?));
      Ok(Value::Message(inner))
    },
    // Second
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.second" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("seconds", Value::I32(value.get_int()?));
      Ok(Value::Message(inner))
    },
    // Time
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.time" => {
      let mut inner = DynamicMessage::new(message_descriptor.clone());
      inner.set_field_by_name("millis", Value::I32(value.get_int()?));
      Ok(Value::Message(inner))
    },
    // Map
    Kind::Message(message_descriptor) if field.is_map() => {
      // Map field equivalent of repeated map entry composed of `key = 1` and `value = 2`.
      k_to_map(value, &message_descriptor)
    },
    // Protobuf message
    Kind::Message(message_descriptor) => Ok(Value::Message(encode_to_message(message_descriptor, value)?)),
    _ => Err("unsupported type\0")
  }
}

/// Encode q dictionary to dynamic message.
fn encode_to_message(message_descriptor: MessageDescriptor, data: K) -> Result<DynamicMessage, &'static str>{
  let mut dynamic_message = DynamicMessage::new(message_descriptor);
  let keys = data.as_mut_slice::<K>()[0].as_mut_slice::<S>();
  let values = data.as_mut_slice::<K>()[1];
  match values.get_type(){
    qtype::BOOL_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<G>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, Value::Bool(*value != 0));
        }             
      }
    },
    qtype::INT_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<I>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, int_to_value(*value, &field)?);
        }             
      }
    },
    qtype::LONG_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<J>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, long_to_value(*value, &field)?);
        }             
      }
    },
    qtype::REAL_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<E>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, Value::F32(*value));
        }             
      }
    },
    qtype::FLOAT_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<F>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, float_to_value(*value, &field)?);
        }             
      }
    },
    qtype::SYMBOL_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<S>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, symbol_to_value(*value, &field)?);
        }             
      }
    },
    qtype::TIMESTAMP_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<J>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, long_to_value(*value, &field)?);
        }             
      }
    },
    qtype::MONTH_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<I>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, int_to_value(*value, &field)?);
        }             
      }
    },
    qtype::DATE_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<I>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, int_to_value(*value, &field)?);
        }             
      }
    },
    qtype::DATETIME_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<F>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, float_to_value(*value, &field)?);
        }             
      }
    },
    qtype::TIMESPAN_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<J>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, long_to_value(*value, &field)?);
        }             
      }
    },
    qtype::MINUTE_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<I>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, int_to_value(*value, &field)?);
        }             
      }
    },
    qtype::SECOND_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<I>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, int_to_value(*value, &field)?);
        }             
      }
    },
    qtype::TIME_LIST => {
      for (key, value) in keys.iter().zip(values.as_mut_slice::<I>()){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
          dynamic_message.set_field(&field, int_to_value(*value, &field)?);
        }             
      }
    },
    qtype::COMPOUND_LIST => {
      let values = values.as_mut_slice::<K>();
      for i in 0 .. keys.len(){
        if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(keys[i])){
          dynamic_message.set_field(&field, k_to_value(values[i], &field)?);
        }
      }
    },
    // There are no other list type
    _ => unreachable!()
  }
  Ok(dynamic_message)
}

//%% Decode %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Decode list of values as q list object and push it to an existing list.
fn decode_list(list: &Vec<Value>, field: &FieldDescriptor, simple: K, compound: &mut K, list_type: &mut i8){
  match field.kind(){
    Kind::Bool => {
      // Bool list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::BOOL_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<G>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_bool().unwrap() as u8;
      });
      unsafe{k(0, str_to_S!("{show x}"), increment_reference_count(*compound), KNULL)};
      compound.push(q_list).unwrap();
    },
    Kind::Int32 | Kind::Sint32 => {
      // Int list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::INT_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<I>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_i32().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Int64 | Kind::Sint64 => {
      // Int list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::LONG_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<J>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_i64().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Float => {
      // Float list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::REAL_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<E>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_f32().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Double => {
      // Int list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::FLOAT_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<F>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_f64().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::String => {
      // List of string
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::COMPOUND_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<K>();
      list.iter().enumerate().for_each(|(i, string)|{
        q_list_slice[i]=new_string(string.as_str().unwrap());
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.symbol" => {
      // Symbol list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::SYMBOL_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<S>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = enumerate(str_to_S!(element.as_message().unwrap().get_field_by_name("symbol").unwrap().as_str().unwrap()));
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.timestamp" => {
      // Timestamp list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::TIMESTAMP_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<J>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.month" => {
      // Month list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::MONTH_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<I>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_message().unwrap().get_field_by_name("months").unwrap().as_i32().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.date" => {
      // Date list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::DATE_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<I>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_message().unwrap().get_field_by_name("days").unwrap().as_i32().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.datetime" => {
      // Month list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::DATETIME_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<F>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_message().unwrap().get_field_by_name("days").unwrap().as_f64().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.timespan" => {
      // Timespan list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::TIMESPAN_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<J>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.minute" => {
      // Minute list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::MINUTE_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<I>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_message().unwrap().get_field_by_name("minutes").unwrap().as_i32().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.second" => {
      // Minute list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::SECOND_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<I>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_message().unwrap().get_field_by_name("seconds").unwrap().as_i32().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) if message_descriptor.full_name() == "q.time" => {
      // Time list
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::TIME_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<I>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_message().unwrap().get_field_by_name("millis").unwrap().as_i32().unwrap();
      });
      compound.push(q_list).unwrap();
    },
    Kind::Message(message_descriptor) => {
      // Protobuf message
      match *list_type{
        qtype::NULL => {
          // Initialize compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          // Move to compound list
          *list_type = qtype::COMPOUND_LIST;
          *compound = simple_to_compound(simple);
        }
      }
      let q_list = new_list(qtype::COMPOUND_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<K>();
      list.iter().enumerate().for_each(|(i, element)|{
        let message = element.as_message().unwrap();
        q_list_slice[i]=decode_message(&message, message_descriptor.fields());
      });
      // Repeated protobuf message is equivalent to repeated dictionary; hence map to table
      compound.push(unsafe{k(0, str_to_S!("{-1 _ x, (::)}"), q_list, KNULL)}).unwrap();
    },
    _ => unimplemented!()
  }
}

/// Decode map of bool key and int compatible value into q dictionary.
fn decode_map_inner_bool_int(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::BOOL_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<G>();
  let values_slice = values.as_mut_slice::<I>();
  match value_type{
    qtype::INT_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_i32().unwrap();
      });
    },
    qtype::MONTH_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_message().unwrap().get_field_by_name("months").unwrap().as_i32().unwrap();
      });
    },
    qtype::DATE_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_message().unwrap().get_field_by_name("days").unwrap().as_i32().unwrap();
      });
    },
    qtype::MINUTE_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_message().unwrap().get_field_by_name("minutes").unwrap().as_i32().unwrap();
      });
    },
    qtype::SECOND_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_message().unwrap().get_field_by_name("seconds").unwrap().as_i32().unwrap();
      });
    },
    qtype::TIME_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_message().unwrap().get_field_by_name("millis").unwrap().as_i32().unwrap();
      });
    },
    // There are no other int compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of bool key and long compatible value into q dictionary.
fn decode_map_inner_bool_long(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::BOOL_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<G>();
  let values_slice = values.as_mut_slice::<J>();
  match value_type{
    qtype::LONG_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_i64().unwrap();
      });
    },
    qtype::TIMESTAMP_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
    },
    qtype::TIMESPAN_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
    },
    // There are no other long compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of bool key and float compatible value into q dictionary.
fn decode_map_inner_bool_float(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::BOOL_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<G>();
  let values_slice = values.as_mut_slice::<F>();
  match value_type{
    qtype::FLOAT_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_f64().unwrap();
      });
    },
    qtype::DATETIME_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_message().unwrap().get_field_by_name("days").unwrap().as_f64().unwrap();
      });
    },
    // There are no other int compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of int key and int compatible value into q dictionary.
fn decode_map_inner_int_int(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::INT_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<I>();
  let values_slice = values.as_mut_slice::<I>();
  match value_type{
    qtype::INT_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_i32().unwrap();
      });
    },
    qtype::MONTH_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("months").unwrap().as_i32().unwrap();
      });
    },
    qtype::DATE_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("days").unwrap().as_i32().unwrap();
      });
    },
    qtype::MINUTE_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("minutes").unwrap().as_i32().unwrap();
      });
    },
    qtype::SECOND_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("seconds").unwrap().as_i32().unwrap();
      });
    },
    qtype::TIME_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("millis").unwrap().as_i32().unwrap();
      });
    },
    // There are no other int compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of int key and long compatible value into q dictionary.
fn decode_map_inner_int_long(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::INT_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<I>();
  let values_slice = values.as_mut_slice::<J>();
  match value_type{
    qtype::LONG_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_i64().unwrap();
      });
    },
    qtype::TIMESTAMP_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
    },
    qtype::TIMESPAN_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
    },
    // There are no other long compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of int key and float compatible value into q dictionary.
fn decode_map_inner_int_float(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::INT_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<I>();
  let values_slice = values.as_mut_slice::<F>();
  match value_type{
    qtype::FLOAT_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_f64().unwrap();
      });
    },
    qtype::DATETIME_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("days").unwrap().as_f64().unwrap();
      });
    },
    // There are no other int compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}


/// Decode map of long key and int compatible value into q dictionary.
fn decode_map_inner_long_int(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::LONG_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<J>();
  let values_slice = values.as_mut_slice::<I>();
  match value_type{
    qtype::INT_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_i32().unwrap();
      });
    },
    qtype::MONTH_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("months").unwrap().as_i32().unwrap();
      });
    },
    qtype::DATE_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("days").unwrap().as_i32().unwrap();
      });
    },
    qtype::MINUTE_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("minutes").unwrap().as_i32().unwrap();
      });
    },
    qtype::SECOND_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("seconds").unwrap().as_i32().unwrap();
      });
    },
    qtype::TIME_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("millis").unwrap().as_i32().unwrap();
      });
    },
    // There are no other int compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of long key and long compatible value into q dictionary.
fn decode_map_inner_long_long(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::LONG_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<J>();
  let values_slice = values.as_mut_slice::<J>();
  match value_type{
    qtype::LONG_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_i64().unwrap();
      });
    },
    qtype::TIMESTAMP_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
    },
    qtype::TIMESPAN_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
    },
    // There are no other long compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of long key and float compatible value into q dictionary.
fn decode_map_inner_long_float(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::LONG_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<J>();
  let values_slice = values.as_mut_slice::<F>();
  match value_type{
    qtype::FLOAT_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_f64().unwrap();
      });
    },
    qtype::DATETIME_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_message().unwrap().get_field_by_name("days").unwrap().as_f64().unwrap();
      });
    },
    // There are no other int compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of symbol key and int compatible value into q dictionary.
fn decode_map_inner_symbol_int(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::SYMBOL_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<S>();
  let values_slice = values.as_mut_slice::<I>();
  match value_type{
    qtype::INT_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_i32().unwrap();
      });
    },
    qtype::MONTH_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_message().unwrap().get_field_by_name("months").unwrap().as_i32().unwrap();
      });
    },
    qtype::DATE_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_message().unwrap().get_field_by_name("days").unwrap().as_i32().unwrap();
      });
    },
    qtype::MINUTE_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_message().unwrap().get_field_by_name("minutes").unwrap().as_i32().unwrap();
      });
    },
    qtype::SECOND_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_message().unwrap().get_field_by_name("seconds").unwrap().as_i32().unwrap();
      });
    },
    qtype::TIME_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_message().unwrap().get_field_by_name("millis").unwrap().as_i32().unwrap();
      });
    },
    // There are no other int compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of symbol key and long compatible value into q dictionary.
fn decode_map_inner_symbol_long(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::SYMBOL_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<S>();
  let values_slice = values.as_mut_slice::<J>();
  match value_type{
    qtype::LONG_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_i64().unwrap();
      });
    },
    qtype::TIMESTAMP_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
    },
    qtype::TIMESPAN_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_message().unwrap().get_field_by_name("nanos").unwrap().as_i64().unwrap();
      });
    },
    // There are no other long compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Decode map of symbol key and float compatible value into q dictionary.
fn decode_map_inner_symbol_float(map: &HashMap<MapKey, Value>, value_type: i8) -> K{
  let keys = new_list(qtype::SYMBOL_LIST, map.len() as i64);
  let values = new_list(value_type, map.len() as i64);
  let keys_slice = keys.as_mut_slice::<S>();
  let values_slice = values.as_mut_slice::<F>();
  match value_type{
    qtype::FLOAT_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_f64().unwrap();
      });
    },
    qtype::DATETIME_LIST => {
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_message().unwrap().get_field_by_name("days").unwrap().as_f64().unwrap();
      });
    },
    // There are no other int compatible type
    _ => unreachable!()
  }
  new_dictionary(keys, values)
}

/// Convert protobuf map into q dictionary.
fn decode_map(map: &HashMap<MapKey, Value>, field: &FieldDescriptor) -> K{
  let kind = field.kind();
  let message_descriptor = kind.as_message().unwrap();
  let value_kind =  message_descriptor.map_entry_value_field().kind();
  match (message_descriptor.map_entry_key_field().kind(), value_kind){
    (Kind::Bool, Kind::Bool) => {
      let keys = new_list(qtype::BOOL_LIST, map.len() as i64);
      let values = new_list(qtype::BOOL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<G>();
      let values_slice = values.as_mut_slice::<G>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_bool().unwrap() as u8;
      });
      new_dictionary(keys, values)
    },
    (Kind::Bool, Kind::Int32 | Kind::Sint32) => decode_map_inner_bool_int(map, qtype::INT_LIST),
    (Kind::Bool, Kind::Int64 | Kind::Sint64) => decode_map_inner_bool_long(map, qtype::LONG_LIST),
    (Kind::Bool, Kind::Float) => {
      let keys = new_list(qtype::BOOL_LIST, map.len() as i64);
      let values = new_list(qtype::REAL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<G>();
      let values_slice = values.as_mut_slice::<E>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = value.as_f32().unwrap();
      });
      new_dictionary(keys, values)
    },
    (Kind::Bool, Kind::Double) => decode_map_inner_bool_float(map, qtype::FLOAT_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.symbol" => {
      let keys = new_list(qtype::BOOL_LIST, map.len() as i64);
      let values = new_list(qtype::SYMBOL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<G>();
      let values_slice = values.as_mut_slice::<S>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = enumerate(str_to_S!(value.as_message().unwrap().get_field_by_name("symbol").unwrap().as_str().unwrap()));
      });
      new_dictionary(keys, values)
    },
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.timestamp" => decode_map_inner_bool_long(map, qtype::TIMESTAMP_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.month" => decode_map_inner_bool_int(map, qtype::MONTH_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.date" => decode_map_inner_bool_int(map, qtype::DATE_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.datetime" => decode_map_inner_bool_float(map, qtype::DATETIME_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.timespan" => decode_map_inner_bool_long(map, qtype::TIMESPAN_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.minute" => decode_map_inner_bool_int(map, qtype::MINUTE_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.second" => decode_map_inner_bool_int(map, qtype::SECOND_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.time" => decode_map_inner_bool_int(map, qtype::TIME_LIST),
    (Kind::Bool, Kind::Message(inner_message_descriptor)) => {
      let keys = new_list(qtype::BOOL_LIST, map.len() as i64);
      let values = new_list(qtype::COMPOUND_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<G>();
      let values_slice = values.as_mut_slice::<K>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_bool().unwrap() as u8;
        values_slice[i] = decode_message(value.as_message().unwrap(), inner_message_descriptor.fields());
      });
      new_dictionary(keys, values)
    },
    (Kind::Int32 | Kind::Sint32, Kind::Bool) => {
      let keys = new_list(qtype::INT_LIST, map.len() as i64);
      let values = new_list(qtype::BOOL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<I>();
      let values_slice = values.as_mut_slice::<G>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_bool().unwrap() as u8;
      });
      new_dictionary(keys, values)
    },
    (Kind::Int32 | Kind::Sint32, Kind::Int32 | Kind::Sint32) => decode_map_inner_int_int(map, qtype::INT_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Int64 | Kind::Sint64) => decode_map_inner_int_long(map, qtype::LONG_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Float) => {
      let keys = new_list(qtype::INT_LIST, map.len() as i64);
      let values = new_list(qtype::REAL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<I>();
      let values_slice = values.as_mut_slice::<E>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = value.as_f32().unwrap();
      });
      new_dictionary(keys, values)
    },
    (Kind::Int32 | Kind::Sint32, Kind::Double) => decode_map_inner_int_float(map, qtype::FLOAT_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.symbol" => {
      let keys = new_list(qtype::INT_LIST, map.len() as i64);
      let values = new_list(qtype::SYMBOL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<I>();
      let values_slice = values.as_mut_slice::<S>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = enumerate(str_to_S!(value.as_message().unwrap().get_field_by_name("symbol").unwrap().as_str().unwrap()));
      });
      new_dictionary(keys, values)
    },
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.timestamp" => decode_map_inner_int_long(map, qtype::TIMESTAMP_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.month" => decode_map_inner_int_int(map, qtype::MONTH_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.date" => decode_map_inner_int_int(map, qtype::DATE_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.datetime" => decode_map_inner_int_float(map, qtype::DATETIME_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.timespan" => decode_map_inner_int_long(map, qtype::TIMESPAN_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.minute" => decode_map_inner_int_int(map, qtype::MINUTE_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.second" => decode_map_inner_int_int(map, qtype::SECOND_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.time" => decode_map_inner_int_int(map, qtype::TIME_LIST),
    (Kind::Int32 | Kind::Sint32, Kind::Message(inner_message_descriptor)) => {
      let keys = new_list(qtype::INT_LIST, map.len() as i64);
      let values = new_list(qtype::COMPOUND_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<I>();
      let values_slice = values.as_mut_slice::<K>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i32().unwrap();
        values_slice[i] = decode_message(value.as_message().unwrap(), inner_message_descriptor.fields());
      });
      new_dictionary(keys, values)
    },
    (Kind::Int64 | Kind::Sint64, Kind::Bool) => {
      let keys = new_list(qtype::LONG_LIST, map.len() as i64);
      let values = new_list(qtype::BOOL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<J>();
      let values_slice = values.as_mut_slice::<G>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_bool().unwrap() as u8;
      });
      new_dictionary(keys, values)
    },
    (Kind::Int64 | Kind::Sint64, Kind::Int32 | Kind::Sint32) => decode_map_inner_long_int(map, qtype::INT_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Int64 | Kind::Sint64) => decode_map_inner_long_long(map, qtype::LONG_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Float) => {
      let keys = new_list(qtype::LONG_LIST, map.len() as i64);
      let values = new_list(qtype::REAL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<J>();
      let values_slice = values.as_mut_slice::<E>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = value.as_f32().unwrap();
      });
      new_dictionary(keys, values)
    },
    (Kind::Int64 | Kind::Sint64, Kind::Double) => decode_map_inner_long_float(map, qtype::FLOAT_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.symbol" => {
      let keys = new_list(qtype::LONG_LIST, map.len() as i64);
      let values = new_list(qtype::SYMBOL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<J>();
      let values_slice = values.as_mut_slice::<S>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = enumerate(str_to_S!(value.as_message().unwrap().get_field_by_name("symbol").unwrap().as_str().unwrap()));
      });
      new_dictionary(keys, values)
    },
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.timestamp" => decode_map_inner_long_long(map, qtype::TIMESTAMP_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.month" => decode_map_inner_long_int(map, qtype::MONTH_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.date" => decode_map_inner_long_int(map, qtype::DATE_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.datetime" => decode_map_inner_long_float(map, qtype::DATETIME_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.timespan" => decode_map_inner_long_long(map, qtype::TIMESPAN_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.minute" => decode_map_inner_long_int(map, qtype::MINUTE_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.second" => decode_map_inner_long_int(map, qtype::SECOND_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.time" => decode_map_inner_long_int(map, qtype::TIME_LIST),
    (Kind::Int64 | Kind::Sint64, Kind::Message(inner_message_descriptor)) => {
      let keys = new_list(qtype::LONG_LIST, map.len() as i64);
      let values = new_list(qtype::COMPOUND_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<J>();
      let values_slice = values.as_mut_slice::<K>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = key.as_i64().unwrap();
        values_slice[i] = decode_message(value.as_message().unwrap(), inner_message_descriptor.fields());
      });
      new_dictionary(keys, values)
    },
    (Kind::String, Kind::Bool) => {
      let keys = new_list(qtype::SYMBOL_LIST, map.len() as i64);
      let values = new_list(qtype::BOOL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<S>();
      let values_slice = values.as_mut_slice::<G>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_bool().unwrap() as u8;
      });
      new_dictionary(keys, values)
    },
    (Kind::String, Kind::Int32 | Kind::Sint32) => decode_map_inner_symbol_int(map, qtype::INT_LIST),
    (Kind::String, Kind::Int64 | Kind::Sint64) => decode_map_inner_symbol_long(map, qtype::LONG_LIST),
    (Kind::String, Kind::Float) => {
      let keys = new_list(qtype::SYMBOL_LIST, map.len() as i64);
      let values = new_list(qtype::REAL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<S>();
      let values_slice = values.as_mut_slice::<E>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = value.as_f32().unwrap();
      });
      new_dictionary(keys, values)
    },
    (Kind::String, Kind::Double) => decode_map_inner_symbol_float(map, qtype::FLOAT_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.symbol" => {
      let keys = new_list(qtype::SYMBOL_LIST, map.len() as i64);
      let values = new_list(qtype::SYMBOL_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<S>();
      let values_slice = values.as_mut_slice::<S>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = enumerate(str_to_S!(value.as_message().unwrap().get_field_by_name("symbol").unwrap().as_str().unwrap()));
      });
      new_dictionary(keys, values)
    },
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.timestamp" => decode_map_inner_symbol_long(map, qtype::TIMESTAMP_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.month" => decode_map_inner_symbol_int(map, qtype::MONTH_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.date" => decode_map_inner_symbol_int(map, qtype::DATE_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.datetime" => decode_map_inner_symbol_float(map, qtype::DATETIME_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.timespan" => decode_map_inner_symbol_long(map, qtype::TIMESPAN_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.minute" => decode_map_inner_symbol_int(map, qtype::MINUTE_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.second" => decode_map_inner_symbol_int(map, qtype::SECOND_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) if inner_message_descriptor.full_name() == "q.time" => decode_map_inner_symbol_int(map, qtype::TIME_LIST),
    (Kind::String, Kind::Message(inner_message_descriptor)) => {
      let keys = new_list(qtype::SYMBOL_LIST, map.len() as i64);
      let values = new_list(qtype::COMPOUND_LIST, map.len() as i64);
      let keys_slice = keys.as_mut_slice::<S>();
      let values_slice = values.as_mut_slice::<K>();
      map.iter().enumerate().for_each(|(i, (key, value))|{
        keys_slice[i] = enumerate(str_to_S!(key.as_str().unwrap()));
        values_slice[i] = decode_message(value.as_message().unwrap(), inner_message_descriptor.fields());
      });
      new_dictionary(keys, values)
    },
    _ => new_error("unsupported type")
  }
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
      // Some value is set to the field
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
        Value::List(list) => {
          // List
          decode_list(list, &field, simple, &mut compound, &mut list_type);
        },
        Value::Map(map) => {
          // Map
          match list_type{
            qtype::NULL =>{
              list_type = qtype::COMPOUND_LIST;
              compound = new_list(qtype::COMPOUND_LIST, 0);
            },
            qtype::COMPOUND_LIST => (),
            _ => {
              // Move to compound list
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
            }
          }
          compound.push(decode_map(map, &field)).unwrap();
        }
        Value::Message(message) => {
          // Protobuf message
          let message_descriptor = message.descriptor();
          let inner_fields =message_descriptor.fields();
          let v = decode_message(message, inner_fields);
          // Move to compound list
          match list_type{
            qtype::NULL =>{
              list_type = qtype::COMPOUND_LIST;
              compound = new_list(qtype::COMPOUND_LIST, 0);
            },
            qtype::COMPOUND_LIST => (),
            _ => {
              list_type = qtype::COMPOUND_LIST;
              compound = simple_to_compound(simple);
            }
          }
          compound.push(v).unwrap();
        },
        _ => unimplemented!()
      }
    }
    else{
      // No value is set to the field. Parse as null.
      // Move to compound list
      match list_type{
        qtype::NULL =>{
          list_type = qtype::COMPOUND_LIST;
          compound = new_list(qtype::COMPOUND_LIST, 0);
        },
        qtype::COMPOUND_LIST => (),
        _ => {
          list_type = qtype::COMPOUND_LIST;
          compound = simple_to_compound(simple);
        }
      }
      compound.push(new_null()).unwrap();
    }
    i += 1;
  });
  match list_type{
    qtype::COMPOUND_LIST => new_dictionary(keys, compound),
    _ => new_dictionary(keys, simple)
  }
}
