use generic_rpc_client::GenericRpcClient;
use tonic::{service::interceptor::InterceptedService, transport::Channel};
use tracing_utils::grpc::InterceptorType;

pub use crate::codegen::generic_rpc::*;
use crate::error::ClientError;

pub async fn connect(
    addr: String,
) -> Result<GenericRpcClient<InterceptedService<Channel, &'static dyn InterceptorType>>, ClientError>
{
    let conn = crate::open_channel(addr, "generic service").await?;

    Ok(GenericRpcClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}
