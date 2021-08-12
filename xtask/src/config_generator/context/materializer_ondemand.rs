use settings_utils::apps::{
    materializer_ondemand::{MaterializerOndemandServicesSettings, MaterializerOndemandSettings},
    LogSettings,
    MonitoringSettings,
};

use crate::config_generator::{
    context::{Context, FromContext},
    defaults::{DEFAULT_OBJECT_BUILDER_HOST, DEFAULT_ON_DEMAND_MATERIALIZER_PORT},
};

pub const MATERIALIZER_ONDEMAND_NAMESPACE: &str = "materializer_ondemand";

impl FromContext for MaterializerOndemandSettings {
    fn from_context(_: &Context) -> anyhow::Result<Self> {
        Ok(Self {
            input_port: DEFAULT_ON_DEMAND_MATERIALIZER_PORT,
            services: MaterializerOndemandServicesSettings {
                object_builder_url: DEFAULT_OBJECT_BUILDER_HOST.to_string(),
            },
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: MATERIALIZER_ONDEMAND_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", MATERIALIZER_ONDEMAND_NAMESPACE),
            },
        })
    }
}
