/// Information to apply to an event.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Application {
    /// Name of a customer.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Event date.
    #[prost(message, optional, tag = "4")]
    pub date: ::core::option::Option<super::q::Date>,
    /// The number of seats to reserve.
    #[prost(int32, tag = "5")]
    pub number: i32,
    /// Class of seat.
    #[prost(enumeration = "Class", tag = "6")]
    pub class: i32,
}
/// Information of a reserved ticket, or a information to cancel a flight.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TicketInfo {
    /// Name of a customer.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Seat ID.
    #[prost(message, repeated, tag = "2")]
    pub seats: ::prost::alloc::vec::Vec<super::q::Symbol>,
    /// Flight date.
    #[prost(message, optional, tag = "3")]
    pub date: ::core::option::Option<super::q::Date>,
}
/// Failure response to a flight request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReservationFailure {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
/// Response to a flight request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Processed {
    #[prost(oneof = "processed::Result", tags = "1, 2")]
    pub result: ::core::option::Option<processed::Result>,
}
/// Nested message and enum types in `Processed`.
pub mod processed {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Result {
        /// Reservation success.
        #[prost(message, tag = "1")]
        Ticket(super::TicketInfo),
        /// Reservation failure.
        #[prost(message, tag = "2")]
        Failure(super::ReservationFailure),
    }
}
/// Message to notify that cancellation was completed.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Cancelled {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
/// Table of available seats by seat class.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AvailableSeats {
    #[prost(map = "string, int32", tag = "1")]
    pub inventory: ::std::collections::HashMap<::prost::alloc::string::String, i32>,
}
/// Class of seat
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Class {
    NoPreference = 0,
    Stand = 1,
    Arena = 2,
    Vip = 3,
}
#[doc = r" Generated client implementations."]
pub mod ticketing_machine_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = " Service to issue a ticket."]
    #[derive(Debug, Clone)]
    pub struct TicketingMachineClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TicketingMachineClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> TicketingMachineClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> TicketingMachineClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            TicketingMachineClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        #[doc = " Customer requests seats and get a result."]
        pub async fn reserve(
            &mut self,
            request: impl tonic::IntoRequest<super::Application>,
        ) -> Result<tonic::Response<super::Processed>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ticket.TicketingMachine/Reserve");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Customer cancels seats."]
        pub async fn cancel(
            &mut self,
            request: impl tonic::IntoRequest<super::TicketInfo>,
        ) -> Result<tonic::Response<super::Cancelled>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/ticket.TicketingMachine/Cancel");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Customer requests current status of the inventory."]
        pub async fn get_available_seats(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> Result<tonic::Response<super::AvailableSeats>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/ticket.TicketingMachine/GetAvailableSeats");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod ticketing_machine_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with TicketingMachineServer."]
    #[async_trait]
    pub trait TicketingMachine: Send + Sync + 'static {
        #[doc = " Customer requests seats and get a result."]
        async fn reserve(
            &self,
            request: tonic::Request<super::Application>,
        ) -> Result<tonic::Response<super::Processed>, tonic::Status>;
        #[doc = " Customer cancels seats."]
        async fn cancel(
            &self,
            request: tonic::Request<super::TicketInfo>,
        ) -> Result<tonic::Response<super::Cancelled>, tonic::Status>;
        #[doc = " Customer requests current status of the inventory."]
        async fn get_available_seats(
            &self,
            request: tonic::Request<()>,
        ) -> Result<tonic::Response<super::AvailableSeats>, tonic::Status>;
    }
    #[doc = " Service to issue a ticket."]
    #[derive(Debug)]
    pub struct TicketingMachineServer<T: TicketingMachine> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: TicketingMachine> TicketingMachineServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for TicketingMachineServer<T>
    where
        T: TicketingMachine,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/ticket.TicketingMachine/Reserve" => {
                    #[allow(non_camel_case_types)]
                    struct ReserveSvc<T: TicketingMachine>(pub Arc<T>);
                    impl<T: TicketingMachine> tonic::server::UnaryService<super::Application> for ReserveSvc<T> {
                        type Response = super::Processed;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Application>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).reserve(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReserveSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ticket.TicketingMachine/Cancel" => {
                    #[allow(non_camel_case_types)]
                    struct CancelSvc<T: TicketingMachine>(pub Arc<T>);
                    impl<T: TicketingMachine> tonic::server::UnaryService<super::TicketInfo> for CancelSvc<T> {
                        type Response = super::Cancelled;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TicketInfo>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).cancel(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CancelSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ticket.TicketingMachine/GetAvailableSeats" => {
                    #[allow(non_camel_case_types)]
                    struct GetAvailableSeatsSvc<T: TicketingMachine>(pub Arc<T>);
                    impl<T: TicketingMachine> tonic::server::UnaryService<()> for GetAvailableSeatsSvc<T> {
                        type Response = super::AvailableSeats;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<()>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_available_seats(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAvailableSeatsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: TicketingMachine> Clone for TicketingMachineServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: TicketingMachine> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: TicketingMachine> tonic::transport::NamedService for TicketingMachineServer<T> {
        const NAME: &'static str = "ticket.TicketingMachine";
    }
}
