use cache::CacheSupplier;
use rpc::schema_registry::Id;
use std::sync::Arc;
use tracing::trace;
use uuid::Uuid;

pub struct InsertDestinationCacheSupplier {
    schema_registry_url: Arc<String>,
}

#[async_trait::async_trait]
impl CacheSupplier<Uuid, String> for InsertDestinationCacheSupplier {
    async fn retrieve(&self, key: Uuid) -> anyhow::Result<String> {
        let schema_registry_url = Arc::clone(&self.schema_registry_url);
        let mut client = rpc::schema_registry::connect(schema_registry_url.to_string()).await?;
        let insert_destination = client
            .get_schema_metadata(Id {
                id: key.to_string(),
            })
            .await?
            .into_inner()
            .insert_destination;

        trace!(
            "Retrieved insert destination for {} from schema registry",
            key
        );

        Ok(insert_destination)
    }
}

impl InsertDestinationCacheSupplier {
    pub fn new(schema_registry_url: Arc<String>) -> Self {
        Self {
            schema_registry_url,
        }
    }
}
