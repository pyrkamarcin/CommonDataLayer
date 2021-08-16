use rpc::{error::ClientError, schema_registry::types::SchemaType};
use warp::{hyper::StatusCode, reject::Reject, Rejection};

#[derive(Debug)]
pub enum Error {
    ClientError(ClientError),
    JsonError(serde_json::Error),
    SingleQueryMissingValue,
    WrongValueFormat,
    ExpectedSchemaType(SchemaType),
    InvalidRepository(String),
    SchemaFetchError(anyhow::Error),
    InvalidSchemaType,
}

impl Reject for Error {}

pub fn recover(rejection: Rejection) -> Result<impl warp::Reply, Rejection> {
    if let Some(error) = rejection.find::<Error>() {
        let message = match error {
            Error::ClientError(err) => err.to_string(),
            Error::JsonError(err) => format!("Unable to serialize JSON: {}", err),
            Error::SingleQueryMissingValue => "Value not returned from query".to_owned(),
            Error::WrongValueFormat => "Value incorrectly formatted".to_owned(),
            Error::ExpectedSchemaType(expected) => {
                format!("This route expects schema type: {:?}", expected)
            }
            Error::InvalidRepository(repository_id) => format!(
                "Repository with id `{}` is not present in config",
                repository_id
            ),
            Error::SchemaFetchError(error) => format!("Failed to fetch schema: {}", error),
            Error::InvalidSchemaType => "Specified schema doesn't match the query kind".to_string(),
        };

        let code = match error {
            Error::ExpectedSchemaType(_)
            | Error::InvalidRepository(_)
            | Error::InvalidSchemaType => StatusCode::BAD_REQUEST,
            Error::ClientError(_)
            | Error::JsonError(_)
            | Error::SingleQueryMissingValue
            | Error::WrongValueFormat
            | Error::SchemaFetchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "message": message })),
            code,
        ))
    } else {
        Err(rejection)
    }
}
