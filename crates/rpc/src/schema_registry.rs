use bb8::{Pool, PooledConnection};
use schema_registry_client::SchemaRegistryClient;
use tower::ServiceBuilder;
use tracing_utils::grpc::{Trace, TraceLayer};

pub use crate::codegen::schema_registry::*;
use crate::error::ClientError;

pub mod types;

pub type SchemaRegistryPool = Pool<SchemaRegistryConnectionManager>;
pub type SchemaRegistryConn = SchemaRegistryClient<Trace>;

pub struct SchemaRegistryConnectionManager {
    pub address: String,
}

pub async fn connect(addr: String) -> Result<SchemaRegistryConn, ClientError> {
    let conn = crate::open_channel(addr, "schema registry").await?;
    let service = ServiceBuilder::new().layer(TraceLayer).service(conn);

    Ok(SchemaRegistryClient::new(service))
}

#[async_trait::async_trait]
impl bb8::ManageConnection for SchemaRegistryConnectionManager {
    type Connection = SchemaRegistryConn;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to schema registry");

        connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.heartbeat(Empty {})
            .await
            .map_err(|source| ClientError::QueryError { source })?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}
