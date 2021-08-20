use general_materializer_client::GeneralMaterializerClient;
use tracing_utils::grpc::{Trace, TraceLayer};

pub use crate::codegen::materializer_general::*;
use crate::error::ClientError;

pub async fn connect(addr: String) -> Result<GeneralMaterializerClient<Trace>, ClientError> {
    let conn = crate::open_channel(addr, "materializer general").await?;
    let service = tower::ServiceBuilder::new().layer(TraceLayer).service(conn);

    Ok(GeneralMaterializerClient::new(service))
}
