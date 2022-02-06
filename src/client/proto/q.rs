#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Symbol {
    #[prost(string, tag = "1")]
    pub symbol: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Timestamp {
    #[prost(int64, tag = "1")]
    pub nanos: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Month {
    #[prost(int32, tag = "1")]
    pub months: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Date {
    #[prost(int32, tag = "1")]
    pub days: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Datetime {
    #[prost(double, tag = "1")]
    pub days: f64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Timespan {
    #[prost(int64, tag = "1")]
    pub nanos: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Minute {
    #[prost(int32, tag = "1")]
    pub minutes: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Second {
    #[prost(int32, tag = "1")]
    pub seconds: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Time {
    #[prost(int32, tag = "1")]
    pub millis: i32,
}
