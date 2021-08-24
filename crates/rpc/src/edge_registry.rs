use bb8::{Pool, PooledConnection};
use edge_registry_client::EdgeRegistryClient;
use tracing_utils::grpc::{Trace, TraceLayer};

pub use crate::codegen::edge_registry::*;
use crate::error::ClientError;

pub mod types;

pub type EdgeRegistryPool = Pool<EdgeRegistryConnectionManager>;
pub type EdgeRegistryConn = EdgeRegistryClient<Trace>;

pub struct EdgeRegistryConnectionManager {
    pub address: String,
}

pub async fn connect(addr: String) -> Result<EdgeRegistryConn, ClientError> {
    let conn = crate::open_channel(addr, "edge registry").await?;
    let service = tower::ServiceBuilder::new().layer(TraceLayer).service(conn);

    Ok(EdgeRegistryClient::new(service))
}

#[async_trait::async_trait]
impl bb8::ManageConnection for EdgeRegistryConnectionManager {
    type Connection = EdgeRegistryConn;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::info!("Connecting to edge registry");

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
