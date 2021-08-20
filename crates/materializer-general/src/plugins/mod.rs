use cdl_dto::materialization::FullView;
pub use postgres::PostgresMaterializer;
use rpc::materializer_general::MaterializedView;
use serde_json::Value;

pub use self::elasticsearch::ElasticsearchMaterializer;

mod elasticsearch;
mod models;
mod postgres;

#[async_trait::async_trait]
pub trait MaterializerPlugin: Send + Sync {
    fn validate_options(&self, options: Value) -> anyhow::Result<()>;
    async fn upsert_view(
        &self,
        view: MaterializedView,
        view_definition: FullView,
    ) -> anyhow::Result<()>;
}
