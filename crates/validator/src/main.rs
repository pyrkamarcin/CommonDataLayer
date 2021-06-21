use lru_cache::LruCache;
use std::sync::{Arc, Mutex};

use crate::settings::Settings;
use metrics_utils as metrics;
use settings_utils::load_settings;
use utils::parallel_task_queue::ParallelTaskQueue;
use validator::Handler;

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

    // let cache = Arc::new(Mutex::new(LruCache::new(settings.cache_capacity)));
    let schema_registry_url = Arc::new(settings.services.schema_registry_url);

    let task_queue = Arc::new(ParallelTaskQueue::default());

    consumer.par_run(Handler::new(schema_registry_url)).await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}
