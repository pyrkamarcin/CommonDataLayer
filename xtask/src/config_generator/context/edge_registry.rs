use crate::config_generator::context::{Communication, Context, FromContext};
use crate::config_generator::defaults::{
    DEFAULT_CDL_NOTIFICATION_CHANNEL, DEFAULT_EDGE_REGISTRY_NOTIFICATION_SOURCE,
    DEFAULT_EDGE_REGISTRY_PORT,
};
use settings_utils::apps::edge_registry::{
    EdgeRegistryAmqpSettings, EdgeRegistryKafkaSettings, EdgeRegistrySettings,
};
use settings_utils::apps::{
    AmqpConsumeOptions, CommunicationMethod, LogSettings, MonitoringSettings, NotificationSettings,
};

pub const EDGE_REGISTRY_NAMESPACE: &str = "edge_registry";

impl FromContext for EdgeRegistrySettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        let mut settings = Self {
            communication_method: CommunicationMethod::Kafka,
            postgres: Default::default(),
            input_port: DEFAULT_EDGE_REGISTRY_PORT,
            kafka: None,
            amqp: None,
            notifications: NotificationSettings {
                enabled: true,
                destination: DEFAULT_CDL_NOTIFICATION_CHANNEL.to_string(),
            },
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: EDGE_REGISTRY_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", EDGE_REGISTRY_NAMESPACE),
            },
        };

        match context.communication {
            Communication::Kafka(ref kafka) => {
                settings.communication_method = CommunicationMethod::Kafka;
                settings.kafka = Some(EdgeRegistryKafkaSettings {
                    brokers: kafka.brokers.clone(),
                    group_id: EDGE_REGISTRY_NAMESPACE.to_string(),
                    ingest_topic: DEFAULT_EDGE_REGISTRY_NOTIFICATION_SOURCE.to_string(),
                });
            }
            Communication::Amqp(ref amqp) => {
                settings.communication_method = CommunicationMethod::Amqp;
                settings.amqp = Some(EdgeRegistryAmqpSettings {
                    exchange_url: amqp.exchange_url.clone(),
                    tag: EDGE_REGISTRY_NAMESPACE.to_string(),
                    ingest_queue: DEFAULT_EDGE_REGISTRY_NOTIFICATION_SOURCE.to_string(),
                    consume_options: Some(AmqpConsumeOptions {
                        no_local: false,
                        no_ack: false,
                        exclusive: false,
                        nowait: false,
                    }),
                });
            }
            Communication::Grpc => settings.communication_method = CommunicationMethod::Grpc,
        }

        settings.postgres = context.postgres.clone().into();

        Ok(settings)
    }
}
