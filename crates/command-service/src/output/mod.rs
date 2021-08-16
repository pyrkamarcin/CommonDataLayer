use cdl_dto::ingestion::BorrowedInsertMessage;
pub use druid::DruidOutputPlugin;
pub use psql::PostgresOutputPlugin;
pub use victoria_metrics::VictoriaMetricsOutputPlugin;

use crate::communication::resolution::Resolution;

mod druid;
mod psql;
mod victoria_metrics;

#[async_trait::async_trait]
pub trait OutputPlugin: Send + Sync + 'static {
    async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Resolution;
    fn name(&self) -> &'static str;
}
