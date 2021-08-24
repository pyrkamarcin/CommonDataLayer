use generic_rpc_client::GenericRpcClient;
use tracing_utils::grpc::{Trace, TraceLayer};

pub use crate::codegen::generic_rpc::*;
use crate::error::ClientError;

pub async fn connect(addr: String) -> Result<GenericRpcClient<Trace>, ClientError> {
    let conn = crate::open_channel(addr, "generic service").await?;
    let service = tower::ServiceBuilder::new().layer(TraceLayer).service(conn);

    Ok(GenericRpcClient::new(service))
}
