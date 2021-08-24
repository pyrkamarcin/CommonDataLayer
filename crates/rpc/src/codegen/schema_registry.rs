#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewSchema {
    #[prost(string, required, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub insert_destination: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub query_address: ::prost::alloc::string::String,
    #[prost(message, required, tag = "4")]
    pub schema_type: SchemaType,
    #[prost(bytes = "vec", required, tag = "5")]
    pub definition: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Schema {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub insert_destination: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub query_address: ::prost::alloc::string::String,
    #[prost(message, required, tag = "5")]
    pub schema_type: SchemaType,
    #[prost(bytes = "vec", required, tag = "6")]
    pub definition: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullSchema {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub insert_destination: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub query_address: ::prost::alloc::string::String,
    #[prost(message, required, tag = "5")]
    pub schema_type: SchemaType,
    #[prost(bytes = "vec", required, tag = "6")]
    pub definition: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "7")]
    pub views: ::prost::alloc::vec::Vec<View>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaUpdate {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub insert_destination: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub query_address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "5")]
    pub schema_type: ::core::option::Option<SchemaType>,
    #[prost(bytes = "vec", optional, tag = "6")]
    pub definition: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct View {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub materializer_address: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "5")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(message, optional, tag = "6")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "7")]
    pub relations: ::prost::alloc::vec::Vec<super::common::Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Filter {
    #[prost(oneof = "filter::FilterKind", tags = "1, 2")]
    pub filter_kind: ::core::option::Option<filter::FilterKind>,
}
/// Nested message and enum types in `Filter`.
pub mod filter {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum FilterKind {
        #[prost(message, tag = "1")]
        Simple(super::SimpleFilter),
        #[prost(message, tag = "2")]
        Complex(super::ComplexFilter),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleFilter {
    #[prost(oneof = "simple_filter::SimpleFilter", tags = "1")]
    pub simple_filter: ::core::option::Option<simple_filter::SimpleFilter>,
}
/// Nested message and enum types in `SimpleFilter`.
pub mod simple_filter {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SimpleFilter {
        #[prost(message, tag = "1")]
        Equals(super::EqualsFilter),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EqualsFilter {
    #[prost(message, required, tag = "1")]
    pub lhs: FilterValue,
    #[prost(message, required, tag = "2")]
    pub rhs: FilterValue,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComplexFilter {
    #[prost(message, required, tag = "1")]
    pub operator: super::common::LogicOperator,
    #[prost(message, repeated, tag = "2")]
    pub operands: ::prost::alloc::vec::Vec<Filter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FilterOperator {
    #[prost(enumeration = "filter_operator::Operator", required, tag = "1")]
    pub operator: i32,
}
/// Nested message and enum types in `FilterOperator`.
pub mod filter_operator {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Operator {
        EqualsOp = 0,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FilterValue {
    #[prost(oneof = "filter_value::FilterValue", tags = "1, 2, 3, 4")]
    pub filter_value: ::core::option::Option<filter_value::FilterValue>,
}
/// Nested message and enum types in `FilterValue`.
pub mod filter_value {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum FilterValue {
        #[prost(message, tag = "1")]
        SchemaField(super::SchemaFieldFilter),
        #[prost(message, tag = "2")]
        ViewPath(super::ViewPathFilter),
        #[prost(message, tag = "3")]
        RawValue(super::RawValueFilter),
        #[prost(message, tag = "4")]
        Computed(super::ComputedFilter),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaFieldFilter {
    #[prost(uint32, required, tag = "1")]
    pub schema_id: u32,
    #[prost(string, required, tag = "2")]
    pub field_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ViewPathFilter {
    #[prost(string, required, tag = "1")]
    pub field_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawValueFilter {
    #[prost(string, required, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComputedFilter {
    #[prost(message, required, tag = "1")]
    pub computation: Computation,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Computation {
    #[prost(oneof = "computation::Computation", tags = "1, 2, 3")]
    pub computation: ::core::option::Option<computation::Computation>,
}
/// Nested message and enum types in `Computation`.
pub mod computation {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Computation {
        #[prost(message, tag = "1")]
        RawValue(super::RawValueComputation),
        #[prost(message, tag = "2")]
        FieldValue(super::FieldValueComputation),
        #[prost(message, tag = "3")]
        EqualsComputation(::prost::alloc::boxed::Box<super::EqualsComputation>),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawValueComputation {
    #[prost(string, required, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FieldValueComputation {
    #[prost(uint32, optional, tag = "1")]
    pub schema_id: ::core::option::Option<u32>,
    #[prost(string, required, tag = "2")]
    pub field_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EqualsComputation {
    #[prost(message, required, boxed, tag = "2")]
    pub lhs: ::prost::alloc::boxed::Box<Computation>,
    #[prost(message, required, boxed, tag = "3")]
    pub rhs: ::prost::alloc::boxed::Box<Computation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullView {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub base_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub materializer_address: ::prost::alloc::string::String,
    #[prost(string, required, tag = "5")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "6")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(message, optional, tag = "7")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "8")]
    pub relations: ::prost::alloc::vec::Vec<super::common::Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewView {
    #[prost(string, optional, tag = "1")]
    pub view_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, required, tag = "2")]
    pub base_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub materializer_address: ::prost::alloc::string::String,
    #[prost(string, required, tag = "5")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "6")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(message, optional, tag = "7")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "8")]
    pub relations: ::prost::alloc::vec::Vec<super::common::Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ViewUpdate {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub materializer_address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, required, tag = "4")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "5")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(bool, required, tag = "6")]
    pub update_fields: bool,
    #[prost(message, optional, tag = "7")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "8")]
    pub relations: ::prost::alloc::vec::Vec<super::common::Relation>,
    #[prost(bool, required, tag = "9")]
    pub update_filters: bool,
    #[prost(bool, required, tag = "10")]
    pub update_relations: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Id {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Schemas {
    #[prost(message, repeated, tag = "1")]
    pub schemas: ::prost::alloc::vec::Vec<Schema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullSchemas {
    #[prost(message, repeated, tag = "1")]
    pub schemas: ::prost::alloc::vec::Vec<FullSchema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaViews {
    #[prost(message, repeated, tag = "1")]
    pub views: ::prost::alloc::vec::Vec<FullView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueToValidate {
    #[prost(string, required, tag = "1")]
    pub schema_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", required, tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Errors {
    #[prost(string, repeated, tag = "1")]
    pub errors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaType {
    #[prost(enumeration = "schema_type::Type", required, tag = "1")]
    pub schema_type: i32,
}
/// Nested message and enum types in `SchemaType`.
pub mod schema_type {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        DocumentStorage = 0,
        Timeseries = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
#[doc = r" Generated client implementations."]
pub mod schema_registry_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct SchemaRegistryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SchemaRegistryClient<tonic::transport::Channel> {
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
    impl<T> SchemaRegistryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + Sync + 'static,
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
        ) -> SchemaRegistryClient<InterceptedService<T, F>>
        where
            F: FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>,
            T: Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            SchemaRegistryClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn add_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::NewSchema>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/AddSchema");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::SchemaUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/UpdateSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_view_to_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::NewView>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/AddViewToSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_view(
            &mut self,
            request: impl tonic::IntoRequest<super::ViewUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/UpdateView");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::Schema>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/GetSchema");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_full_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::FullSchema>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetFullSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_view(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::FullView>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/GetView");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_all_schemas(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::Schemas>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetAllSchemas",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_all_full_schemas(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::FullSchemas>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetAllFullSchemas",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_all_views_of_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::SchemaViews>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetAllViewsOfSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_all_views_by_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::SchemaViews>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetAllViewsByRelation",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_base_schema_of_view(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::Schema>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetBaseSchemaOfView",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn validate_value(
            &mut self,
            request: impl tonic::IntoRequest<super::ValueToValidate>,
        ) -> Result<tonic::Response<super::Errors>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/ValidateValue",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn watch_all_schema_updates(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::Schema>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/WatchAllSchemaUpdates",
            );
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn heartbeat(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/Heartbeat");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod schema_registry_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with SchemaRegistryServer."]
    #[async_trait]
    pub trait SchemaRegistry: Send + Sync + 'static {
        async fn add_schema(
            &self,
            request: tonic::Request<super::NewSchema>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status>;
        async fn update_schema(
            &self,
            request: tonic::Request<super::SchemaUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn add_view_to_schema(
            &self,
            request: tonic::Request<super::NewView>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status>;
        async fn update_view(
            &self,
            request: tonic::Request<super::ViewUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn get_schema(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::Schema>, tonic::Status>;
        async fn get_full_schema(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::FullSchema>, tonic::Status>;
        async fn get_view(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::FullView>, tonic::Status>;
        async fn get_all_schemas(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Schemas>, tonic::Status>;
        async fn get_all_full_schemas(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::FullSchemas>, tonic::Status>;
        async fn get_all_views_of_schema(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaViews>, tonic::Status>;
        async fn get_all_views_by_relation(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaViews>, tonic::Status>;
        async fn get_base_schema_of_view(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::Schema>, tonic::Status>;
        async fn validate_value(
            &self,
            request: tonic::Request<super::ValueToValidate>,
        ) -> Result<tonic::Response<super::Errors>, tonic::Status>;
        #[doc = "Server streaming response type for the WatchAllSchemaUpdates method."]
        type WatchAllSchemaUpdatesStream: futures_core::Stream<Item = Result<super::Schema, tonic::Status>>
            + Send
            + Sync
            + 'static;
        async fn watch_all_schema_updates(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<Self::WatchAllSchemaUpdatesStream>, tonic::Status>;
        async fn heartbeat(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct SchemaRegistryServer<T: SchemaRegistry> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: SchemaRegistry> SchemaRegistryServer<T> {
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
            F: FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> Service<http::Request<B>> for SchemaRegistryServer<T>
    where
        T: SchemaRegistry,
        B: Body + Send + Sync + 'static,
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
                "/schema_registry.SchemaRegistry/AddSchema" => {
                    #[allow(non_camel_case_types)]
                    struct AddSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::NewSchema> for AddSchemaSvc<T> {
                        type Response = super::Id;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NewSchema>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddSchemaSvc(inner);
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
                "/schema_registry.SchemaRegistry/UpdateSchema" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::SchemaUpdate> for UpdateSchemaSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SchemaUpdate>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateSchemaSvc(inner);
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
                "/schema_registry.SchemaRegistry/AddViewToSchema" => {
                    #[allow(non_camel_case_types)]
                    struct AddViewToSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::NewView> for AddViewToSchemaSvc<T> {
                        type Response = super::Id;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NewView>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_view_to_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddViewToSchemaSvc(inner);
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
                "/schema_registry.SchemaRegistry/UpdateView" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateViewSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::ViewUpdate> for UpdateViewSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ViewUpdate>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_view(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateViewSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetSchemaSvc<T> {
                        type Response = super::Schema;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetSchemaSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetFullSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetFullSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetFullSchemaSvc<T> {
                        type Response = super::FullSchema;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_full_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetFullSchemaSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetView" => {
                    #[allow(non_camel_case_types)]
                    struct GetViewSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetViewSvc<T> {
                        type Response = super::FullView;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_view(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetViewSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetAllSchemas" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllSchemasSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for GetAllSchemasSvc<T> {
                        type Response = super::Schemas;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_all_schemas(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAllSchemasSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetAllFullSchemas" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllFullSchemasSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for GetAllFullSchemasSvc<T> {
                        type Response = super::FullSchemas;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_all_full_schemas(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAllFullSchemasSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetAllViewsOfSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllViewsOfSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetAllViewsOfSchemaSvc<T> {
                        type Response = super::SchemaViews;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_all_views_of_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAllViewsOfSchemaSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetAllViewsByRelation" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllViewsByRelationSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetAllViewsByRelationSvc<T> {
                        type Response = super::SchemaViews;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_all_views_by_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetAllViewsByRelationSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetBaseSchemaOfView" => {
                    #[allow(non_camel_case_types)]
                    struct GetBaseSchemaOfViewSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetBaseSchemaOfViewSvc<T> {
                        type Response = super::Schema;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_base_schema_of_view(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBaseSchemaOfViewSvc(inner);
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
                "/schema_registry.SchemaRegistry/ValidateValue" => {
                    #[allow(non_camel_case_types)]
                    struct ValidateValueSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::ValueToValidate>
                        for ValidateValueSvc<T>
                    {
                        type Response = super::Errors;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValueToValidate>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).validate_value(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ValidateValueSvc(inner);
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
                "/schema_registry.SchemaRegistry/WatchAllSchemaUpdates" => {
                    #[allow(non_camel_case_types)]
                    struct WatchAllSchemaUpdatesSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::ServerStreamingService<super::Empty>
                        for WatchAllSchemaUpdatesSvc<T>
                    {
                        type Response = super::Schema;
                        type ResponseStream = T::WatchAllSchemaUpdatesStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).watch_all_schema_updates(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WatchAllSchemaUpdatesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/Heartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct HeartbeatSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for HeartbeatSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).heartbeat(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HeartbeatSvc(inner);
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
    impl<T: SchemaRegistry> Clone for SchemaRegistryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: SchemaRegistry> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: SchemaRegistry> tonic::transport::NamedService for SchemaRegistryServer<T> {
        const NAME: &'static str = "schema_registry.SchemaRegistry";
    }
}
