//! This module provides serialization and deserialization around protobuf map message.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use std::collections::HashMap;
use prost_reflect::{Value, MessageDescriptor, FieldDescriptor, Kind, MapKey};
use kdbplus::qtype;
use kdbplus::api::*;
use super::{int_to_value, long_to_value, float_to_value, symbol_to_value, k_to_value, decode_message};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Private Functions
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% Encode %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Convert q dictionary to protobuf map type specified by a given message descriptor.
pub(crate) fn k_to_map(value: K, message_descriptor: &MessageDescriptor) -> Result<Value, &'static str>{
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

//%% Decode %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/


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
pub(crate) fn decode_map(map: &HashMap<MapKey, Value>, field: &FieldDescriptor) -> K{
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
