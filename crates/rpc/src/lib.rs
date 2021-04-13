pub mod common {
    tonic::include_proto!("common");
}
pub mod edge_registry;
pub mod error;
pub mod generic;
pub mod materializer;
pub mod object_builder;
pub mod query_service;
pub mod query_service_ts;
pub mod schema_registry;

pub use tonic;
