use settings_utils::apps::{
    data_router::{
        DataRouterAmqpSettings,
        DataRouterConsumerKafkaSettings,
        DataRouterGRpcSettings,
        DataRouterServicesSettings,
        DataRouterSettings,
    },
    AmqpConsumeOptions,
    CommunicationMethod,
    LogSettings,
    MonitoringSettings,
};

use crate::config_generator::{
    context::{Communication, Context, FromContext},
    defaults::{
        DEFAULT_DATA_ROUTER_INGEST_SOURCE,
        DEFAULT_DATA_ROUTER_LISTEN_URL,
        DEFAULT_SCHEMA_REGISTRY_HOST,
    },
};

pub const DATA_ROUTER_NAMESPACE: &str = "data_router";

impl FromContext for DataRouterSettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        let mut settings = Self {
            communication_method: CommunicationMethod::Kafka,
            cache_capacity: 1024,
            async_task_limit: 32,
            kafka: None,
            amqp: None,
            grpc: None,
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: DATA_ROUTER_NAMESPACE.to_string(),
            },
            services: DataRouterServicesSettings {
                schema_registry_url: DEFAULT_SCHEMA_REGISTRY_HOST.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", DATA_ROUTER_NAMESPACE),
            },
            repositories: Default::default(),
        };

        match context.communication {
            Communication::Kafka(ref kafka) => {
                settings.communication_method = CommunicationMethod::Kafka;
                settings.kafka = Some(DataRouterConsumerKafkaSettings {
                    brokers: kafka.brokers.clone(),
                    group_id: DATA_ROUTER_NAMESPACE.to_string(),
                    ingest_topic: DEFAULT_DATA_ROUTER_INGEST_SOURCE.to_string(),
                })
            }
            Communication::Amqp(ref amqp) => {
                settings.communication_method = CommunicationMethod::Amqp;
                settings.amqp = Some(DataRouterAmqpSettings {
                    exchange_url: amqp.exchange_url.clone(),
                    tag: DATA_ROUTER_NAMESPACE.to_string(),
                    ingest_queue: DEFAULT_DATA_ROUTER_INGEST_SOURCE.to_string(),
                    consume_options: Some(AmqpConsumeOptions {
                        no_local: false,
                        no_ack: false,
                        exclusive: false,
                        nowait: false,
                    }),
                })
            }
            Communication::Grpc => {
                settings.communication_method = CommunicationMethod::Grpc;
                settings.grpc = Some(DataRouterGRpcSettings {
                    address: DEFAULT_DATA_ROUTER_LISTEN_URL.parse()?,
                })
            }
        }

        Ok(settings)
    }
}
