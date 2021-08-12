use object_builder_client::ObjectBuilderClient;
use tonic::{service::interceptor::InterceptedService, transport::Channel};
use tracing_utils::grpc::InterceptorType;

pub use crate::codegen::object_builder::*;
use crate::error::ClientError;

pub async fn connect(
    addr: String,
) -> Result<
    ObjectBuilderClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    ClientError,
> {
    let conn = crate::open_channel(addr, "object builder").await?;

    Ok(ObjectBuilderClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}
