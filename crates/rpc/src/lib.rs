mod codegen;
pub use crate::codegen::common;

pub mod edge_registry;
pub mod error;
pub mod generic;
pub mod materializer_general;
pub mod materializer_ondemand;
pub mod object_builder;
pub mod query_service;
pub mod query_service_ts;
pub mod schema_registry;

use std::{error::Error, io};

use error::ClientError;
pub use tonic;
use tonic::transport::{Channel, TimeoutExpired};

pub async fn open_channel(
    addr: String,
    service_name: &'static str,
) -> Result<Channel, ClientError> {
    let endpoint = tonic::transport::Endpoint::new(addr)
        .map_err(|err| ClientError::ConnectionError { source: err })?
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(5));

    endpoint.connect().await.map_err(|err| {
        eprintln!(
            "Failed to connect to endpoint: {} - {:?}",
            err,
            err.source()
        );

        if err
            .source()
            .map(|s| s.is::<TimeoutExpired>())
            .unwrap_or(false)
        {
            return ClientError::TimeoutError {
                service: service_name,
            };
        }

        match err.source().and_then(|s| s.downcast_ref::<io::Error>()) {
            Some(ioe) if ioe.kind() == io::ErrorKind::TimedOut => {
                return ClientError::TimeoutError {
                    service: service_name,
                };
            }
            _ => {}
        }

        ClientError::ConnectionError { source: err }
    })
}
