use std::collections::HashMap;

use futures_util::TryStreamExt;
use rpc::{
    schema_registry::types::SchemaType,
    tonic::codegen::{http::header::CONTENT_TYPE, Arc},
};
use settings_utils::apps::RepositoryStaticRouting;
use uuid::Uuid;

use crate::{
    error::Error,
    routes,
    routes::APPLICATION_JSON,
    schema::{SchemaCache, SchemaMetadata},
};

#[tracing::instrument(skip(cache))]
pub async fn query_by_schema(
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let SchemaMetadata {
        query_address,
        schema_type,
    } = routes::get_routing_info(schema_id, repository_id, cache, routing).await?;

    match &schema_type {
        SchemaType::DocumentStorage => {
            let values: HashMap<_, _> =
                rpc::query_service::query_by_schema(schema_id.to_string(), query_address.clone())
                    .await
                    .map_err(Error::ClientError)?
                    .map_ok(|o| (o.object_id, o.payload))
                    .try_collect()
                    .await
                    .map_err(Error::ClientError)?;

            Ok(warp::reply::with_header(
                serde_json::to_vec(&routes::byte_map_to_json_map(values)?)
                    .map_err(Error::JsonError)?,
                CONTENT_TYPE,
                APPLICATION_JSON,
            ))
        }
        SchemaType::Timeseries => {
            let timeseries = rpc::query_service_ts::query_by_schema(
                schema_id.to_string(),
                query_address.clone(),
            )
            .await
            .map_err(Error::ClientError)?;
            Ok(warp::reply::with_header(
                timeseries.into_bytes(),
                CONTENT_TYPE,
                APPLICATION_JSON,
            ))
        }
    }
}
