use anyhow::bail;
use settings_utils::apps::{
    query_service_ts::{
        QueryServiceTsDruidSettings,
        QueryServiceTsRepositoryKind,
        QueryServiceTsSettings,
        QueryServiceTsVictoriaMetricsSettings,
    },
    LogSettings,
    MonitoringSettings,
};

use crate::config_generator::{
    context::{Context, FromContext, Repo},
    defaults::DEFAULT_QUERY_SERVICE_TS_PORT,
};

pub const QUERY_SERVICE_TS_NAMESPACE: &str = "query_service_ts";

impl FromContext for QueryServiceTsSettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        let mut settings = Self {
            repository_kind: QueryServiceTsRepositoryKind::VictoriaMetrics,
            input_port: DEFAULT_QUERY_SERVICE_TS_PORT,
            druid: None,
            victoria_metrics: None,
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: QUERY_SERVICE_TS_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", QUERY_SERVICE_TS_NAMESPACE),
            },
        };

        match context.repo {
            Repo::Postgres => bail!("postgres is not a valid timeseries repo backend"),
            Repo::VictoriaMetrics(ref vm) => {
                settings.repository_kind = QueryServiceTsRepositoryKind::VictoriaMetrics;
                settings.victoria_metrics = Some(QueryServiceTsVictoriaMetricsSettings {
                    url: vm.url.parse()?,
                });
            }
            Repo::Druid(ref druid) => {
                settings.repository_kind = QueryServiceTsRepositoryKind::Druid;
                settings.druid = Some(QueryServiceTsDruidSettings {
                    url: druid.url.clone(),
                    table_name: druid.table_name.clone(),
                })
            }
        }

        Ok(settings)
    }
}
