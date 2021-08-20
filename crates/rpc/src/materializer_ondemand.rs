use bb8::{Pool, PooledConnection};
use on_demand_materializer_client::OnDemandMaterializerClient;
use tracing_utils::grpc::{Trace, TraceLayer};

pub use crate::codegen::materializer_ondemand::*;
use crate::error::ClientError;

pub type OnDemandMaterializerConn = OnDemandMaterializerClient<Trace>;
pub type OnDemandMaterializerPool = Pool<OnDemandMaterializerConnectionManager>;

pub struct OnDemandMaterializerConnectionManager {
    pub address: String,
}

pub async fn connect(addr: String) -> Result<OnDemandMaterializerConn, ClientError> {
    let conn = crate::open_channel(addr, "materializer on-demand").await?;
    let service = tower::ServiceBuilder::new().layer(TraceLayer).service(conn);

    Ok(OnDemandMaterializerClient::new(service))
}

#[async_trait::async_trait]
impl bb8::ManageConnection for OnDemandMaterializerConnectionManager {
    type Connection = OnDemandMaterializerConn;
    type Error = ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to object builder");

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
