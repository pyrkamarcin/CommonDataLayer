use object_builder_client::ObjectBuilderClient;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing_utils::grpc::InterceptorType;

pub use crate::codegen::object_builder::*;
use crate::error::ClientError;

pub async fn connect(
    addr: impl Into<String>,
) -> Result<
    ObjectBuilderClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    ClientError,
> {
    connect_inner(addr.into())
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<
    ObjectBuilderClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    tonic::transport::Error,
> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(ObjectBuilderClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}
