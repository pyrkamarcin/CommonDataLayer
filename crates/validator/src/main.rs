use crate::settings::Settings;
use metrics_utils as metrics;
use settings_utils::load_settings;
use std::sync::Arc;
use validator::{Handler, Validator};

mod settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    metrics::serve(&settings.monitoring);

    let consumer = settings.consumer().await?;
    let producer = Arc::new(settings.producer().await?);

    let validator = Validator::new(
        settings.cache_capacity,
        settings.services.schema_registry_url,
    );

    consumer
        .par_run(Handler::new(validator, producer, settings.send_to))
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}
