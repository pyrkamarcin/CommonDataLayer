pub use crate::codegen::query_service_ts::*;
use crate::error::ClientError;
use query_service_ts_client::QueryServiceTsClient;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use tracing_utils::grpc::InterceptorType;

pub async fn connect(
    addr: String,
) -> Result<
    QueryServiceTsClient<InterceptedService<Channel, &'static dyn InterceptorType>>,
    ClientError,
> {
    let conn = crate::open_channel(addr, "query service (timeseries)").await?;

    Ok(QueryServiceTsClient::with_interceptor(
        conn,
        &tracing_utils::grpc::interceptor,
    ))
}

pub async fn query_by_range(
    schema_id: String,
    object_id: String,
    start: String,
    end: String,
    step: String,
    addr: String,
) -> Result<String, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_range(Range {
            schema_id,
            object_id,
            start,
            end,
            step,
        })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().timeseries)
}

pub async fn query_by_schema(schema_id: String, addr: String) -> Result<String, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_by_schema(SchemaId { schema_id })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().timeseries)
}

pub async fn query_raw(raw_statement: String, addr: String) -> Result<Vec<u8>, ClientError> {
    let mut conn = connect(addr).await?;
    let response = conn
        .query_raw(RawStatement { raw_statement })
        .await
        .map_err(|err| ClientError::QueryError { source: err })?;

    Ok(response.into_inner().value_bytes)
}
