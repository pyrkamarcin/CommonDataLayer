use crate::report;
use thiserror::Error as DeriveError;
use tokio::sync::oneshot::error::RecvError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Failed to send message")]
    FailedToSend,
    #[error("There was issue inserting data into the database")]
    UnableToInsert,
    #[error("Channel was closed on sender side")]
    SenderDropped,
    #[error("Failed to send failure report `{0}`")]
    ReportingError(report::Error),
}

impl From<RecvError> for Error {
    fn from(_: RecvError) -> Self {
        Self::UnableToInsert
    }
}
