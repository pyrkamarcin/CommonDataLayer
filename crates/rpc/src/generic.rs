pub use crate::codegen::generic_rpc::*;
use crate::error::ClientError;
use generic_rpc_client::GenericRpcClient;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing_utils::grpc::InterceptorType;

pub async fn connect(
    addr: String,
) -> Result<GenericRpcClient<InterceptedService<Channel, &'static dyn InterceptorType>>, ClientError>
{
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<
    GenericRpcClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    tonic::transport::Error,
> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(GenericRpcClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}
