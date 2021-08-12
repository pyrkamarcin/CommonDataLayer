use settings_utils::apps::{
    query_router::{QueryRouterServicesSettings, QueryRouterSettings},
    LogSettings,
    MonitoringSettings,
};

use crate::config_generator::{
    context::{Context, FromContext},
    defaults::{DEFAULT_QUERY_ROUTER_PORT, DEFAULT_SCHEMA_REGISTRY_HOST},
};

pub const QUERY_ROUTER_NAMESPACE: &str = "query_router";

impl FromContext for QueryRouterSettings {
    fn from_context(_: &Context) -> anyhow::Result<Self> {
        Ok(Self {
            cache_capacity: 1024,
            input_port: DEFAULT_QUERY_ROUTER_PORT,
            services: QueryRouterServicesSettings {
                schema_registry_url: DEFAULT_SCHEMA_REGISTRY_HOST.to_string(),
            },
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: QUERY_ROUTER_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", QUERY_ROUTER_NAMESPACE),
            },
            repositories: Default::default(),
        })
    }
}
