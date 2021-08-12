use std::net::{Ipv4Addr, SocketAddrV4};

use anyhow::Context;
use metrics_utils as metrics;
use query_service_ts::{druid::DruidQuery, victoria::VictoriaQuery};
use rpc::query_service_ts::query_service_ts_server::{QueryServiceTs, QueryServiceTsServer};
use settings_utils::{
    apps::query_service_ts::{QueryServiceTsRepositoryKind, QueryServiceTsSettings},
    *,
};
use tonic::transport::Server;

//Could be extracted to utils, dunno how without schema
async fn spawn_server<Q: QueryServiceTs>(service: Q, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .trace_fn(tracing_utils::grpc::trace_fn)
        .add_service(QueryServiceTsServer::new(service))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: QueryServiceTsSettings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");
    metrics::serve(&settings.monitoring);

    match settings.repository_kind {
        QueryServiceTsRepositoryKind::VictoriaMetrics => {
            spawn_server(
                VictoriaQuery::load(settings.victoria_metrics.expect("victoria_metrics config"))
                    .await?,
                settings.input_port,
            )
            .await
        }
        QueryServiceTsRepositoryKind::Druid => {
            spawn_server(
                DruidQuery::load(settings.druid.expect("druid config")).await?,
                settings.input_port,
            )
            .await
        }
    }
}
