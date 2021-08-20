use object_builder_client::ObjectBuilderClient;
use tracing_utils::grpc::{Trace, TraceLayer};

pub use crate::codegen::object_builder::*;
use crate::error::ClientError;

pub async fn connect(addr: String) -> Result<ObjectBuilderClient<Trace>, ClientError> {
    let conn = crate::open_channel(addr, "object builder").await?;
    let service = tower::ServiceBuilder::new().layer(TraceLayer).service(conn);

    Ok(ObjectBuilderClient::new(service))
}
