use anyhow::bail;
use settings_utils::apps::{
    partial_update_engine::{
        PartialUpdateEngineKafkaSettings,
        PartialUpdateEngineNotificationConsumerSettings,
        PartialUpdateEngineServicesSettings,
        PartialUpdateEngineSettings,
    },
    CommunicationMethod,
    LogSettings,
    MonitoringSettings,
};

use crate::config_generator::{
    context::{Communication, Context, FromContext},
    defaults::{
        DEFAULT_CDL_NOTIFICATION_CHANNEL,
        DEFAULT_PUE_EGEST_TOPIC,
        DEFAULT_SCHEMA_REGISTRY_HOST,
    },
};

pub const PARTIAL_UPDATE_ENGINE_NAMESPACE: &str = "partial_update_engine";

impl FromContext for PartialUpdateEngineSettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        let mut settings = Self {
            communication_method: CommunicationMethod::Kafka,
            sleep_phase_length: 15,
            kafka: PartialUpdateEngineKafkaSettings {
                brokers: "".to_string(),
                egest_topic: DEFAULT_PUE_EGEST_TOPIC.to_string(),
            },
            notification_consumer: PartialUpdateEngineNotificationConsumerSettings {
                brokers: "".to_string(),
                group_id: PARTIAL_UPDATE_ENGINE_NAMESPACE.to_string(),
                source: DEFAULT_CDL_NOTIFICATION_CHANNEL.to_string(),
            },
            services: PartialUpdateEngineServicesSettings {
                schema_registry_url: DEFAULT_SCHEMA_REGISTRY_HOST.to_string(),
            },
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: PARTIAL_UPDATE_ENGINE_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", PARTIAL_UPDATE_ENGINE_NAMESPACE),
            },
        };

        match context.communication {
            Communication::Kafka(ref kafka) => {
                settings.communication_method = CommunicationMethod::Kafka;
                settings.kafka.brokers = kafka.brokers.clone();
                settings.notification_consumer.brokers = kafka.brokers.clone();
            }
            _ => bail!("partial update engine supports only Kafka environments"),
        }

        Ok(settings)
    }
}
