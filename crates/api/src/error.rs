use communication_utils::Error as MessagingError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to parse UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Invalid schema type. Expected `0` or `1` but found `{0}`")]
    InvalidSchemaType(i32),
    #[error("Unable to connect to data router publisher: {0}")]
    PublisherError(MessagingError),
    #[error("Error while parsing view fields: {0}")]
    ViewFieldError(serde_json::Error),
    #[error("Could not process query")]
    QueryError,
}

impl async_graphql::ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_err, e| {
            tracing_utils::graphql::inject_span(_err, e);
        })
    }
}

impl Error {
    pub fn query<E: std::error::Error>(err: E) -> Self {
        tracing::error!(?err, "Error during query");
        Self::QueryError
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
