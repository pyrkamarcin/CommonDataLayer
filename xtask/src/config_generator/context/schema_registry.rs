use settings_utils::apps::{
    schema_registry::{
        SchemaRegistryAmqpSettings,
        SchemaRegistryKafkaSettings,
        SchemaRegistryServicesSettings,
        SchemaRegistrySettings,
    },
    CommunicationMethod,
    LogSettings,
    MonitoringSettings,
};

use crate::config_generator::{
    context::{Communication, Context, FromContext},
    defaults::{DEFAULT_EDGE_REGISTRY_HOST, DEFAULT_SCHEMA_REGISTRY_PORT},
};

pub const SCHEMA_REGISTRY_NAMESPACE: &str = "schema_registry";

impl FromContext for SchemaRegistrySettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        let mut settings = Self {
            communication_method: CommunicationMethod::Kafka,
            input_port: DEFAULT_SCHEMA_REGISTRY_PORT,
            import_file: None,
            export_dir: None,
            services: SchemaRegistryServicesSettings {
                edge_registry_url: DEFAULT_EDGE_REGISTRY_HOST.to_string(),
            },
            postgres: Default::default(),
            kafka: None,
            amqp: None,
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: SCHEMA_REGISTRY_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", SCHEMA_REGISTRY_NAMESPACE),
            },
        };

        match context.communication {
            Communication::Kafka(ref kafka) => {
                settings.communication_method = CommunicationMethod::Kafka;
                settings.kafka = Some(SchemaRegistryKafkaSettings {
                    brokers: kafka.brokers.clone(),
                });
            }
            Communication::Amqp(ref amqp) => {
                settings.communication_method = CommunicationMethod::Amqp;
                settings.amqp = Some(SchemaRegistryAmqpSettings {
                    exchange_url: amqp.exchange_url.clone(),
                });
            }
            Communication::Grpc => settings.communication_method = CommunicationMethod::Grpc,
        }

        settings.postgres = context.postgres.clone().into();

        Ok(settings)
    }
}
