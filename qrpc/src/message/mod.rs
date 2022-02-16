//! This module provides serialization and deserialization between protobuf message and q object.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

mod list;
mod map;

use std::iter::ExactSizeIterator;
use std::result::Result;
use once_cell::sync::Lazy;
use bytes::Bytes;
use prost::Message;
use prost_reflect::{DynamicMessage, FileDescriptor, Value, ReflectMessage, MessageDescriptor, FieldDescriptor, Kind};
use kdbplus::qtype;
use kdbplus::api::*;
use kdbplus::api::native::k;
use list::decode_list;
use map::{k_to_map, decode_map};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Global Variables
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Bytes representing compiled files.
const PROTO_FILE_DESCRIPTOR_SET_BYTES: &[u8] = include_bytes!("../../qrpc_fd_set");
/// File descriptor of compiled files.
pub(crate) static PROTO_FILE_DESCRIPTOR: Lazy<FileDescriptor> = Lazy::new(|| FileDescriptor::decode(PROTO_FILE_DESCRIPTOR_SET_BYTES).unwrap());

//++++++++++++++++++++++++++++++++++++++++++++++++++//
//>> Interface
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
        // Enum
        Kind::Enum(enum_descriptor) => {
            let range = &enum_descriptor.enum_descriptor_proto().value;
            if range[0].number.unwrap() <= value as i32 && range[range.len()-1].number.unwrap() >= value as i32{
                Ok(Value::EnumNumber(value as i32))
            }
            else{
                Err("not a reserved enum value\0")
            }
        }
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
                // Names of enum fields
                let enum_sources_ = message_descriptor.fields().filter_map(|field|{
                    match field.kind(){
                        Kind::Enum(enum_descriptor) => Some(enum_descriptor.name().to_string()),
                        _ => None
                    }
                }).collect::<Vec<_>>();
                let enum_sources = enum_sources_.iter().map(|source| source.as_str()).collect::<Vec<_>>();
                Ok(Value::List((0..value.len() as usize).into_iter().map(|i|{
                    // Can I get enum field name??
                    let row = value.get_row(i, &enum_sources).unwrap();
                    let encoded = Value::Message(encode_to_message(message_descriptor.clone(), row)?);
                    decrement_reference_count(row);
                    Ok(encoded)
                }).collect::<Result<Vec<Value>, &'static str>>()?))
            }
            else{
                Err("type mismatch. expected: table\0")
            }
        },
        // Enum
        Kind::Enum(enum_descriptor) if field.is_list() => {
            // Enum list
            if value.get_type() == qtype::ENUM_LIST{
                let range = &enum_descriptor.enum_descriptor_proto().value;
                let start = range[0].number.unwrap();
                let end = range[range.len()-1].number.unwrap();
                Ok(Value::List(value.as_mut_slice::<J>().iter().map(|value|{
                    if start <= *value as i32 && end >= *value as i32{
                        Ok(Value::EnumNumber(*value as i32))
                    }
                    else{
                        Err("not a reserved enum value\0")
                    }
                }).collect::<Result<Vec<Value>, &'static str>>()?))
            }
            else{
                Err("type mismatch. expected: time list\0")
            }
        }
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
        // Enum
        Kind::Enum(enum_descriptor) => {
            let index = value.get_long().unwrap() as i32;
            let range = &enum_descriptor.enum_descriptor_proto().value;
            if range[0].number.unwrap() <= index && range[range.len()-1].number.unwrap() >= index{
                Ok(Value::EnumNumber(index))
            }
            else{
                Err("not a reserved enum value\0")
            }
        }
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
pub(crate) fn encode_to_message(message_descriptor: MessageDescriptor, data: K) -> Result<DynamicMessage, &'static str>{
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
        qtype::ENUM_LIST => {
            for (key, value) in keys.iter().zip(values.as_mut_slice::<J>()){
                if let Some(field) = dynamic_message.descriptor().get_field_by_name(S_to_str(*key)){
                    dynamic_message.set_field(&field, long_to_value(*value, &field)?);
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

/// Convert dynamic message into q dictionary.
pub(crate) fn decode_message(dynamic_message: &DynamicMessage, fields: impl ExactSizeIterator<Item = FieldDescriptor>) -> K{
    let mut keys = new_list(qtype::SYMBOL_LIST, 0);
    let mut simple = KNULL;
    let mut compound = KNULL;
    let mut list_type = qtype::NULL;
    let mut enum_source = String::new();
    let mut i = 0;
    fields.into_iter().for_each(|field|{
        if dynamic_message.has_field(&field){
            // Oneof field is populated.
            // Store field name as a key
            keys.push_symbol(field.name()).unwrap();
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                simple.push_raw(*v).unwrap();
                            },
                            qtype::REAL_LIST => {
                                simple.push_raw(*v).unwrap();
                            },
                            qtype::COMPOUND_LIST => {
                                compound.push(new_real(*v as f64)).unwrap();
                            },
                            _ => {
                                // Move to compound list
                                list_type = qtype::COMPOUND_LIST;
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
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
                                compound = simple_to_compound(simple, "");
                                compound.push(new_time(v.as_i32().unwrap())).unwrap();
                            }
                        }
                    },
                    Value::List(list) => {
                        // List
                        decode_list(list, &field, simple, &mut compound, &mut list_type, &enum_source);
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
                                compound = simple_to_compound(simple, "");
                            }
                        }
                        compound.push(decode_map(map, &field)).unwrap();
                    }
                    Value::Message(message) => {
                        // Protobuf message
                        let message_descriptor = message.descriptor();
                        let inner_fields = message_descriptor.fields();
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
                                compound = simple_to_compound(simple, "");
                            }
                        }
                        compound.push(v).unwrap();
                    },
                    Value::EnumNumber(index) => {
                        // Enum
                        match list_type{
                            qtype::NULL =>{
                                enum_source = get_enum_name(&field).unwrap();
                                list_type = qtype::ENUM_LIST;
                                simple = new_list(qtype::ENUM_LIST, 0);
                                simple.push_raw(*index as i64).unwrap();
                            },
                            qtype::ENUM_LIST => {
                                let enum_name = get_enum_name(&field).unwrap();
                                // It is assured that enum_source is not empty when list type is enum list
                                //    by the first case.
                                if enum_source != enum_name{
                                    // Enum list from two different sources is a compound list
                                    list_type = qtype::COMPOUND_LIST;
                                    compound = simple_to_compound(simple, &enum_source);
                                    compound.push(new_enum(enum_name.as_str(), *index as i64)).unwrap();
                                }
                                else{
                                    // Same enum source
                                    simple.push_raw(*index as i64).unwrap();
                                }
                            }
                            qtype::COMPOUND_LIST => {
                                compound.push(new_enum(&get_enum_name(&field).unwrap(), *index as i64)).unwrap();
                            },
                            _ => {
                                // Move to compound list
                                list_type = qtype::COMPOUND_LIST;
                                compound = simple_to_compound(simple, &enum_source);
                                compound.push(new_enum(&get_enum_name(&field).unwrap(), *index as i64)).unwrap();
                            }
                        }
                    }
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
                        compound = simple_to_compound(simple, "");
                    }
                }
                compound.push(new_null()).unwrap();
            }
            i += 1;
        }
    });
    match list_type{
        qtype::COMPOUND_LIST => new_dictionary(keys, compound),
        qtype::ENUM_LIST => {
            // Enum list itself cannot be returned to q due to lack of link to enum sources
            let function = format!("{{`{}${} x}}", enum_source, enum_source);
            // Move to long list to pass to source enum
            let indices = new_list(qtype::LONG_LIST, simple.len());
            indices.as_mut_slice::<J>().copy_from_slice(&simple.as_mut_slice::<J>());
            decrement_reference_count(simple);
            let new_simple = unsafe{k(0, str_to_S!(function), indices, KNULL)};
            new_dictionary(keys, new_simple)
        }
        _ => new_dictionary(keys, simple)
    }
}

//%% Utility %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Get underlying enum name from a field descriptor with prefix `.grpc.package.`.
fn get_enum_name(field_descriptor: &FieldDescriptor) -> Option<String>{
    match field_descriptor.kind(){
        Kind::Enum(enum_descriptor) => Some(format!(".grpc.{}",enum_descriptor.full_name().to_string())),
        _ => None
    }
}
