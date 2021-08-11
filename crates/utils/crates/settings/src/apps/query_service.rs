use crate::apps::{LogSettings, MonitoringSettings, PostgresSettings};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryServiceSettings {
    pub input_port: u16,

    pub postgres: PostgresSettings,
    pub monitoring: MonitoringSettings,

    #[serde(default)]
    pub log: LogSettings,
}
