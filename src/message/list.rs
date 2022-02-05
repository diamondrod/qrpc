//! This module provides serialization and deserialization around protobuf repeated message.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use prost_reflect::{Value, FieldDescriptor, Kind};
use kdbplus::qtype;
use kdbplus::api::*;
use kdbplus::api::native::k;
use super::decode_message;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Private Functions
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Decode list of values as q list object and push it to an existing list.
pub(crate) fn decode_list(list: &Vec<Value>, field: &FieldDescriptor, simple: K, compound: &mut K, list_type: &mut i8, enum_source: &str){
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
          *compound = simple_to_compound(simple, enum_source);
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
    Kind::Enum(enum_descriptor) => {
      // Enum
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
          *compound = simple_to_compound(simple, enum_source);
        }
      }
      let q_list = new_list(qtype::LONG_LIST, list.len() as i64);
      let q_list_slice = q_list.as_mut_slice::<J>();
      list.iter().enumerate().for_each(|(i, element)|{
        q_list_slice[i] = element.as_enum_number().unwrap() as i64;
      });
      let enum_name = enum_descriptor.name();
      let sym = unsafe{k(0, str_to_S!(enum_name), KNULL)};
      if sym.get_type() == qtype::ERROR{
        // Not defined yet.
        // Get all enum values
        let values = &enum_descriptor.enum_descriptor_proto().value.iter().map(|v| v.name.as_ref()).collect::<Vec<_>>();
        let enum_values = new_list(qtype::SYMBOL_LIST, values.len() as i64);
        let enum_values_slice = enum_values.as_mut_slice::<S>();
        values.iter().enumerate().for_each(|(i, value)|{
          enum_values_slice[i] = enumerate(str_to_S!(value.unwrap()));
        });
        // Set values to sym
        let function = format!("set[{}]", enum_name);
        unsafe{k(0, str_to_S!(function), enum_values)};
      }

      // Free no longer necessary value
      decrement_reference_count(sym);

      let function = format!("{{`{}${} x}}", enum_name, enum_name);
      compound.push(unsafe{k(0, str_to_S!(function), q_list, KNULL)}).unwrap();
    }
    _ => unimplemented!()
  }
}
