use rdkafka::error::KafkaError;
use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Failed to create producer `{0}`")]
    ProducerCreation(rdkafka::error::KafkaError),
    #[error("Channel was closed on sender side.")]
    SenderDropped,
    #[error("Failed to deliver Kafka report")]
    FailedToReport(KafkaError),
}
