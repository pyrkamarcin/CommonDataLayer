pub use crate::codegen::materializer_ondemand::*;
use crate::error::ClientError;
use bb8::{Pool, PooledConnection};
use on_demand_materializer_client::OnDemandMaterializerClient;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing_utils::grpc::InterceptorType;

pub type OnDemandMaterializerConn =
    OnDemandMaterializerClient<InterceptedService<Channel, &'static dyn InterceptorType>>;
pub type OnDemandMaterializerPool = Pool<OnDemandMaterializerConnectionManager>;

pub struct OnDemandMaterializerConnectionManager {
    pub address: String,
}

pub async fn connect(addr: String) -> Result<OnDemandMaterializerConn, ClientError> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(addr: String) -> Result<OnDemandMaterializerConn, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(OnDemandMaterializerClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
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
