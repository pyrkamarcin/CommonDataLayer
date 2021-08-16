use thiserror::Error as DeriveError;
pub use tonic::{Code, Status};

#[derive(Debug, DeriveError)]
pub enum ClientError {
    #[error("Failed to connect to: {source}")]
    ConnectionError { source: tonic::transport::Error },
    #[error("Timed out trying to connect to {service}. Is the service up?")]
    TimeoutError { service: &'static str },
    #[error("Error during query: {source}")]
    QueryError { source: Status },
}
