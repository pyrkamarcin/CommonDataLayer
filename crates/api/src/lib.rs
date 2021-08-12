use communication_utils::publisher::CommonPublisher;
use settings_utils::apps::{api::ApiSettings, CommunicationMethod};

pub mod error;
pub mod events;
pub mod schema;
pub mod types;

pub async fn publisher(settings: &ApiSettings) -> anyhow::Result<CommonPublisher> {
    match (
        &settings.kafka,
        &settings.amqp,
        &settings.communication_method,
    ) {
        (Some(kafka), _, CommunicationMethod::Kafka) => {
            Ok(CommonPublisher::new_kafka(&kafka.brokers).await?)
        }
        (_, Some(amqp), CommunicationMethod::Amqp) => {
            Ok(CommonPublisher::new_amqp(&amqp.exchange_url).await?)
        }
        (_, _, CommunicationMethod::Grpc) => Ok(CommonPublisher::new_grpc().await?),
        _ => anyhow::bail!("Unsupported consumer specification"),
    }
}
