use general_materializer_client::GeneralMaterializerClient;
use tonic::{service::interceptor::InterceptedService, transport::Channel};
use tracing_utils::grpc::InterceptorType;

pub use crate::codegen::materializer_general::*;
use crate::error::ClientError;

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
