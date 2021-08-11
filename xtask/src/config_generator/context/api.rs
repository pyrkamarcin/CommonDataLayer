use crate::config_generator::context::{Communication, Context, FromContext};
use crate::config_generator::defaults::{
    DEFAULT_API_PORT, DEFAULT_CDL_NOTIFICATION_CHANNEL, DEFAULT_DATA_ROUTER_INGEST_SOURCE,
    DEFAULT_EDGE_REGISTRY_HOST, DEFAULT_ON_DEMAND_MATERIALIZER_HOST, DEFAULT_QUERY_ROUTER_HOST,
    DEFAULT_SCHEMA_REGISTRY_HOST,
};
use settings_utils::apps::api::{
    ApiAmqpSettings, ApiKafkaSettings, ApiNotificationConsumerSettings, ApiServiceSettings,
    ApiSettings,
};
use settings_utils::apps::{CommunicationMethod, LogSettings};

pub const API_NAMESPACE: &str = "api";

impl FromContext for ApiSettings {
    fn from_context(context: &Context) -> anyhow::Result<ApiSettings> {
        let mut settings = Self {
            communication_method: CommunicationMethod::Kafka,
            input_port: DEFAULT_API_PORT,
            insert_destination: DEFAULT_DATA_ROUTER_INGEST_SOURCE.to_string(),
            kafka: None,
            amqp: None,
            services: ApiServiceSettings {
                schema_registry_url: DEFAULT_SCHEMA_REGISTRY_HOST.to_string(),
                edge_registry_url: DEFAULT_EDGE_REGISTRY_HOST.to_string(),
                on_demand_materializer_url: DEFAULT_ON_DEMAND_MATERIALIZER_HOST.to_string(),
                query_router_url: DEFAULT_QUERY_ROUTER_HOST.to_string(),
            },
            notification_consumer: Some(ApiNotificationConsumerSettings {
                source: DEFAULT_CDL_NOTIFICATION_CHANNEL.to_string(),
            }),
            log: LogSettings {
                rust_log: format!("info,{}=debug", API_NAMESPACE),
            },
        };

        match context.communication {
            Communication::Kafka(ref kafka) => {
                settings.communication_method = CommunicationMethod::Kafka;
                settings.kafka = Some(ApiKafkaSettings {
                    brokers: kafka.brokers.clone(),
                    group_id: API_NAMESPACE.to_string(),
                })
            }
            Communication::Amqp(ref amqp) => {
                settings.communication_method = CommunicationMethod::Amqp;
                settings.amqp = Some(ApiAmqpSettings {
                    exchange_url: amqp.exchange_url.clone(),
                    tag: API_NAMESPACE.to_string(),
                })
            }
            Communication::Grpc => settings.communication_method = CommunicationMethod::Grpc,
        }
        Ok(settings)
    }
}
