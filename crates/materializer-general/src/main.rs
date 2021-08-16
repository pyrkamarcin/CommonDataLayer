#![feature(async_closure)]

use std::sync::Arc;

use materializer_general::MaterializerImpl;
use rpc::materializer_general::general_materializer_server::GeneralMaterializerServer;
use settings_utils::{apps::materializer_general::MaterializerGeneralSettings, load_settings};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: MaterializerGeneralSettings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    utils::status_endpoints::serve(&settings.monitoring);
    metrics_utils::serve(&settings.monitoring);

    let notification_publisher = settings
        .notifications
        .publisher(
            async || settings.publisher().await,
            "Kafka".to_string(),
            "MaterializerGeneral",
        )
        .await?;

    let materializer = MaterializerImpl::new(
        settings.postgres,
        Arc::new(notification_publisher),
        settings.services.schema_registry_url,
        settings.cache_capacity,
    )
    .await?;

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .trace_fn(tracing_utils::grpc::trace_fn)
        .add_service(GeneralMaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], settings.input_port).into())
        .await?;

    Ok(())
}
