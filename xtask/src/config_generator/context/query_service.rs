use crate::config_generator::context::{Context, FromContext, Repo};
use crate::config_generator::defaults::DEFAULT_QUERY_SERVICE_PORT;
use anyhow::bail;
use settings_utils::apps::query_service::QueryServiceSettings;
use settings_utils::apps::{LogSettings, MonitoringSettings};

pub const QUERY_SERVICE_NAMESPACE: &str = "query_service";

impl FromContext for QueryServiceSettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        if let Repo::Postgres = context.repo {
            Ok(Self {
                postgres: context.postgres.clone().into(),
                input_port: DEFAULT_QUERY_SERVICE_PORT,
                monitoring: MonitoringSettings {
                    metrics_port: 0,
                    status_port: 0,
                    otel_service_name: QUERY_SERVICE_NAMESPACE.to_string(),
                },
                log: LogSettings {
                    rust_log: format!("info,{}=debug", QUERY_SERVICE_NAMESPACE),
                },
            })
        } else {
            bail!("unsupported repository kind")
        }
    }
}
