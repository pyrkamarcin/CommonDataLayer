use bb8::{Pool, PooledConnection};
use edge_registry_client::EdgeRegistryClient;
use tonic::{service::interceptor::InterceptedService, transport::Channel};
use tracing_utils::grpc::InterceptorType;

pub use crate::codegen::edge_registry::*;
use crate::error::ClientError;

pub type EdgeRegistryPool = Pool<EdgeRegistryConnectionManager>;
pub type EdgeRegistryConn =
    EdgeRegistryClient<InterceptedService<Channel, &'static dyn InterceptorType>>;

pub struct EdgeRegistryConnectionManager {
    pub address: String,
}

pub async fn connect(addr: String) -> Result<EdgeRegistryConn, ClientError> {
    let conn = crate::open_channel(addr, "edge registry").await?;

    Ok(EdgeRegistryClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}

#[async_trait::async_trait]
impl bb8::ManageConnection for EdgeRegistryConnectionManager {
    type Connection = EdgeRegistryConn;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to edge registry");

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
