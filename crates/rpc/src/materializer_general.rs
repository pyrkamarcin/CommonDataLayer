use general_materializer_client::GeneralMaterializerClient;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing_utils::grpc::InterceptorType;

pub use crate::codegen::materializer_general::*;
use crate::error::ClientError;

pub async fn connect(
    addr: String,
) -> Result<
    GeneralMaterializerClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    ClientError,
> {
    connect_inner(addr)
        .await
        .map_err(|err| ClientError::ConnectionError { source: err })
}

async fn connect_inner(
    addr: String,
) -> Result<
    GeneralMaterializerClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    tonic::transport::Error,
> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(GeneralMaterializerClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}
