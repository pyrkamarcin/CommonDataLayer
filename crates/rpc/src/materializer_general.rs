pub use crate::codegen::materializer_general::*;
use crate::error::ClientError;
use general_materializer_client::GeneralMaterializerClient;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing_utils::grpc::InterceptorType;

pub async fn connect(
    addr: String,
) -> Result<
    GeneralMaterializerClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    ClientError,
> {
    let conn = crate::open_channel(addr, "materializer general").await?;

    Ok(GeneralMaterializerClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}
