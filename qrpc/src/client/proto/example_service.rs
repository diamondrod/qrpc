/// Message representing an order.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Order {
    #[prost(int32, tag = "1")]
    pub table: i32,
    #[prost(enumeration = "Menu", repeated, tag = "2")]
    pub items: ::prost::alloc::vec::Vec<i32>,
    #[prost(message, optional, tag = "3")]
    pub ordered_time: ::core::option::Option<super::q::Timestamp>,
}
/// Message representing acceptance.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Acceptance {
    #[prost(bool, tag = "1")]
    pub accepted: bool,
    #[prost(string, tag = "2")]
    pub reason: ::prost::alloc::string::String,
}
/// Message representing an expense with table ID.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Expense {
    #[prost(int32, tag = "1")]
    pub table: i32,
}
/// Message representing order history.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct History {
    #[prost(message, optional, tag = "1")]
    pub time: ::core::option::Option<super::q::Timestamp>,
    #[prost(enumeration = "Menu", tag = "2")]
    pub item: i32,
    #[prost(int64, tag = "3")]
    pub unit: i64,
    #[prost(float, tag = "4")]
    pub price: f32,
}
/// Message representing a total due.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Total {
    #[prost(message, repeated, tag = "1")]
    pub history: ::prost::alloc::vec::Vec<History>,
    #[prost(float, tag = "2")]
    pub total: f32,
}
/// Available menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Menu {
    Smile = 0,
    Pizza = 1,
    Spaghetti = 2,
    Salad = 3,
    Steak = 4,
    Sushi = 5,
    Hamburger = 6,
    Chips = 7,
    Coke = 8,
}
#[doc = r" Generated client implementations."]
pub mod restaurant_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = " Service mocking a restaurant order system."]
    #[derive(Debug, Clone)]
    pub struct RestaurantClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RestaurantClient<tonic::transport::Channel> {
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
    impl<T> RestaurantClient<T>
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
        ) -> RestaurantClient<InterceptedService<T, F>>
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
            RestaurantClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn submit(
            &mut self,
            request: impl tonic::IntoRequest<super::Order>,
        ) -> Result<tonic::Response<super::Acceptance>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/example_service.Restaurant/Submit");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn finish(
            &mut self,
            request: impl tonic::IntoRequest<super::Expense>,
        ) -> Result<tonic::Response<super::Total>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/example_service.Restaurant/Finish");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod restaurant_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with RestaurantServer."]
    #[async_trait]
    pub trait Restaurant: Send + Sync + 'static {
        async fn submit(
            &self,
            request: tonic::Request<super::Order>,
        ) -> Result<tonic::Response<super::Acceptance>, tonic::Status>;
        async fn finish(
            &self,
            request: tonic::Request<super::Expense>,
        ) -> Result<tonic::Response<super::Total>, tonic::Status>;
    }
    #[doc = " Service mocking a restaurant order system."]
    #[derive(Debug)]
    pub struct RestaurantServer<T: Restaurant> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Restaurant> RestaurantServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for RestaurantServer<T>
    where
        T: Restaurant,
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
                "/example_service.Restaurant/Submit" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitSvc<T: Restaurant>(pub Arc<T>);
                    impl<T: Restaurant> tonic::server::UnaryService<super::Order> for SubmitSvc<T> {
                        type Response = super::Acceptance;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Order>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).submit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubmitSvc(inner);
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
                "/example_service.Restaurant/Finish" => {
                    #[allow(non_camel_case_types)]
                    struct FinishSvc<T: Restaurant>(pub Arc<T>);
                    impl<T: Restaurant> tonic::server::UnaryService<super::Expense> for FinishSvc<T> {
                        type Response = super::Total;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Expense>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).finish(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FinishSvc(inner);
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
    impl<T: Restaurant> Clone for RestaurantServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Restaurant> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Restaurant> tonic::transport::NamedService for RestaurantServer<T> {
        const NAME: &'static str = "example_service.Restaurant";
    }
}
