use communication_utils::publisher::CommonPublisher;
use serde::{Deserialize, Serialize};

use crate::apps::{LogSettings, MonitoringSettings, NotificationSettings, PostgresSettings};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MaterializerGeneralSettings {
    pub input_port: u16,
    pub cache_capacity: usize,

    pub postgres: PostgresSettings,
    pub kafka: Option<MaterializerGeneralKafkaSettings>,
    #[serde(default)]
    pub notifications: NotificationSettings,

    pub monitoring: MonitoringSettings,

    #[serde(default)]
    pub log: LogSettings,

    pub services: MaterializerGeneralServicesSettings,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MaterializerGeneralKafkaSettings {
    pub brokers: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MaterializerGeneralServicesSettings {
    pub schema_registry_url: String,
}

impl MaterializerGeneralSettings {
    pub async fn publisher(&self) -> anyhow::Result<CommonPublisher> {
        if let Some(kafka) = &self.kafka {
            Ok(CommonPublisher::new_kafka(&kafka.brokers).await?)
        } else {
            anyhow::bail!("Missing Kafka config for notifications")
        }
    }
}
