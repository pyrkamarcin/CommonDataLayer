use anyhow::bail;
use settings_utils::apps::{
    object_builder::{
        ObjectBuilderAmqpSettings,
        ObjectBuilderKafkaSettings,
        ObjectBuilderServicesSettings,
        ObjectBuilderSettings,
    },
    AmqpConsumeOptions,
    CommunicationMethod,
    LogSettings,
    MonitoringSettings,
    NotificationSettings,
};

use crate::config_generator::{
    context::{Communication, Context, FromContext},
    defaults::{
        DEFAULT_CDL_NOTIFICATION_CHANNEL,
        DEFAULT_EDGE_REGISTRY_HOST,
        DEFAULT_OBJECT_BUILDER_PORT,
        DEFAULT_PUE_EGEST_TOPIC,
        DEFAULT_SCHEMA_REGISTRY_HOST,
    },
};

pub const OBJECT_BUILDER_NAMESPACE: &str = "object_builder";

impl FromContext for ObjectBuilderSettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        let mut settings = Self {
            communication_method: CommunicationMethod::Kafka,
            input_port: DEFAULT_OBJECT_BUILDER_PORT,
            chunk_capacity: 128,
            kafka: None,
            amqp: None,
            services: ObjectBuilderServicesSettings {
                schema_registry_url: DEFAULT_SCHEMA_REGISTRY_HOST.to_string(),
                edge_registry_url: DEFAULT_EDGE_REGISTRY_HOST.to_string(),
            },
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: OBJECT_BUILDER_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", OBJECT_BUILDER_NAMESPACE),
            },
            notifications: NotificationSettings {
                enabled: true,
                destination: DEFAULT_CDL_NOTIFICATION_CHANNEL.to_string(),
            },
        };

        match context.communication {
            Communication::Kafka(ref kafka) => {
                settings.communication_method = CommunicationMethod::Kafka;
                settings.kafka = Some(ObjectBuilderKafkaSettings {
                    brokers: kafka.brokers.clone(),
                    group_id: OBJECT_BUILDER_NAMESPACE.to_string(),
                    ingest_topic: DEFAULT_PUE_EGEST_TOPIC.to_string(),
                });
            }
            Communication::Amqp(ref amqp) => {
                settings.communication_method = CommunicationMethod::Amqp;
                settings.amqp = Some(ObjectBuilderAmqpSettings {
                    exchange_url: amqp.exchange_url.clone(),
                    tag: OBJECT_BUILDER_NAMESPACE.to_string(),
                    ingest_queue: DEFAULT_PUE_EGEST_TOPIC.to_string(),
                    consume_options: Some(AmqpConsumeOptions {
                        no_local: false,
                        no_ack: false,
                        exclusive: false,
                        nowait: false,
                    }),
                })
            }
            Communication::Grpc => bail!("grpc communication in not supported in object builder"),
        }

        Ok(settings)
    }
}
