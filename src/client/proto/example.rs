/// Message composed of atom types.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Atoms {
    #[prost(bool, tag = "1")]
    pub bool_f: bool,
    #[prost(int32, tag = "2")]
    pub int_f: i32,
    #[prost(int64, tag = "3")]
    pub long_f: i64,
    #[prost(float, tag = "4")]
    pub real_f: f32,
    #[prost(double, tag = "5")]
    pub float_f: f64,
    #[prost(message, optional, tag = "6")]
    pub symbol_f: ::core::option::Option<super::q::Symbol>,
    #[prost(message, optional, tag = "7")]
    pub timestamp_f: ::core::option::Option<super::q::Timestamp>,
    #[prost(message, optional, tag = "8")]
    pub month_f: ::core::option::Option<super::q::Month>,
    #[prost(message, optional, tag = "9")]
    pub date_f: ::core::option::Option<super::q::Date>,
    #[prost(message, optional, tag = "10")]
    pub datetime_f: ::core::option::Option<super::q::Datetime>,
    #[prost(message, optional, tag = "11")]
    pub timespan_f: ::core::option::Option<super::q::Timespan>,
    #[prost(message, optional, tag = "12")]
    pub minute_f: ::core::option::Option<super::q::Minute>,
    #[prost(message, optional, tag = "13")]
    pub second_f: ::core::option::Option<super::q::Second>,
    #[prost(message, optional, tag = "14")]
    pub time_f: ::core::option::Option<super::q::Time>,
}
/// Message composed of list types.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Lists {
    #[prost(bool, repeated, tag = "1")]
    pub bool_f: ::prost::alloc::vec::Vec<bool>,
    #[prost(int32, repeated, tag = "2")]
    pub int_f: ::prost::alloc::vec::Vec<i32>,
    #[prost(int64, repeated, tag = "3")]
    pub long_f: ::prost::alloc::vec::Vec<i64>,
    #[prost(float, repeated, tag = "4")]
    pub real_f: ::prost::alloc::vec::Vec<f32>,
    #[prost(double, repeated, tag = "5")]
    pub float_f: ::prost::alloc::vec::Vec<f64>,
    #[prost(message, repeated, tag = "6")]
    pub symbol_f: ::prost::alloc::vec::Vec<super::q::Symbol>,
    #[prost(message, repeated, tag = "7")]
    pub timestamp_f: ::prost::alloc::vec::Vec<super::q::Timestamp>,
    #[prost(message, repeated, tag = "8")]
    pub month_f: ::prost::alloc::vec::Vec<super::q::Month>,
    #[prost(message, repeated, tag = "9")]
    pub date_f: ::prost::alloc::vec::Vec<super::q::Date>,
    #[prost(message, repeated, tag = "10")]
    pub datetime_f: ::prost::alloc::vec::Vec<super::q::Datetime>,
    #[prost(message, repeated, tag = "11")]
    pub timespan_f: ::prost::alloc::vec::Vec<super::q::Timespan>,
    #[prost(message, repeated, tag = "12")]
    pub minute_f: ::prost::alloc::vec::Vec<super::q::Minute>,
    #[prost(message, repeated, tag = "13")]
    pub second_f: ::prost::alloc::vec::Vec<super::q::Second>,
    #[prost(message, repeated, tag = "14")]
    pub time_f: ::prost::alloc::vec::Vec<super::q::Time>,
}
/// Inner message contained in `Outer`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Inner {
    #[prost(int64, tag = "1")]
    pub inner_muscle: i64,
    #[prost(message, optional, tag = "2")]
    pub inner_mind: ::core::option::Option<super::q::Symbol>,
}
/// Nested message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Outer {
    #[prost(bool, tag = "1")]
    pub out_law: bool,
    #[prost(message, optional, tag = "2")]
    pub inner: ::core::option::Option<Inner>,
}
/// Table row.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Row {
    #[prost(message, optional, tag = "1")]
    pub host: ::core::option::Option<super::q::Symbol>,
    #[prost(sint32, tag = "2")]
    pub port: i32,
    #[prost(message, optional, tag = "3")]
    pub running: ::core::option::Option<super::q::Timespan>,
    #[prost(string, tag = "4")]
    pub user: ::prost::alloc::string::String,
}
/// Message containing a table.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Table {
    #[prost(message, repeated, tag = "1")]
    pub rows: ::prost::alloc::vec::Vec<Row>,
}
/// Message composed of maps.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Mappy {
    #[prost(map = "string, int32", tag = "1")]
    pub id: ::std::collections::HashMap<::prost::alloc::string::String, i32>,
    #[prost(map = "int64, message", tag = "2")]
    pub xday: ::std::collections::HashMap<i64, super::q::Month>,
    #[prost(map = "bool, message", tag = "3")]
    pub physical: ::std::collections::HashMap<bool, Inner>,
}
/// Message containing oneof field.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OneOf {
    #[prost(bool, tag = "1")]
    pub r#static: bool,
    #[prost(oneof = "one_of::Random", tags = "2, 3, 4, 5")]
    pub random: ::core::option::Option<one_of::Random>,
}
/// Nested message and enum types in `OneOf`.
pub mod one_of {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Random {
        #[prost(int32, tag = "2")]
        IntF(i32),
        #[prost(string, tag = "3")]
        StringF(::prost::alloc::string::String),
        #[prost(message, tag = "4")]
        MonthF(super::super::q::Month),
        #[prost(message, tag = "5")]
        SymbolF(super::super::q::Symbol),
    }
}
/// Message holding enum values.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Basket {
    #[prost(enumeration = "Fruit", repeated, tag = "1")]
    pub desserts: ::prost::alloc::vec::Vec<i32>,
    #[prost(double, tag = "2")]
    pub price: f64,
    #[prost(enumeration = "Vegetable", tag = "3")]
    pub snack: i32,
}
/// Message representing available fruit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Fruit {
    RottenFruit = 0,
    Apple = 1,
    Banana = 2,
    Citrus = 3,
    DragonFruit = 4,
}
/// Message representing available vegetables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Vegetable {
    RottenVegetable = 0,
    Tomato = 1,
    Cabage = 2,
    Mashroom = 3,
}
