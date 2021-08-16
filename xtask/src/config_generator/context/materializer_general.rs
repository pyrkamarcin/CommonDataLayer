use anyhow::bail;
use settings_utils::apps::{
    materializer_general::{
        MaterializerGeneralKafkaSettings,
        MaterializerGeneralServicesSettings,
        MaterializerGeneralSettings,
    },
    LogSettings,
    MonitoringSettings,
    NotificationSettings,
};

use crate::config_generator::{
    context::{Communication, Context, FromContext},
    defaults::{
        DEFAULT_CDL_NOTIFICATION_CHANNEL,
        DEFAULT_GENERAL_MATERIALIZER_PORT,
        DEFAULT_SCHEMA_REGISTRY_HOST,
    },
};

pub const MATERIALIZER_GENERAL_NAMESPACE: &str = "materializer_general";

impl FromContext for MaterializerGeneralSettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        Ok(Self {
            postgres: context.postgres.clone().into(),
            kafka: match context.communication {
                Communication::Kafka(ref kafka) => Some(MaterializerGeneralKafkaSettings {
                    brokers: kafka.brokers.to_string(),
                }),
                _ => bail!("materializer general supports only Kafka environments"),
            },
            input_port: DEFAULT_GENERAL_MATERIALIZER_PORT,
            cache_capacity: 1024,
            notifications: NotificationSettings {
                enabled: true,
                destination: DEFAULT_CDL_NOTIFICATION_CHANNEL.to_string(),
            },
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: MATERIALIZER_GENERAL_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", MATERIALIZER_GENERAL_NAMESPACE),
            },
            services: MaterializerGeneralServicesSettings {
                schema_registry_url: DEFAULT_SCHEMA_REGISTRY_HOST.to_string(),
            },
        })
    }
}
