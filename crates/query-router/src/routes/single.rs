use std::collections::HashMap;

use futures_util::TryStreamExt;
use rpc::{
    schema_registry::types::SchemaType,
    tonic::codegen::{http::header::CONTENT_TYPE, Arc},
};
use serde::{Deserialize, Serialize};
use settings_utils::apps::RepositoryStaticRouting;
use uuid::Uuid;

use crate::{
    error::Error,
    routes,
    routes::APPLICATION_JSON,
    schema::{SchemaCache, SchemaMetadata},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    from: String,
    to: String,
    step: String,
}

#[tracing::instrument(skip(cache))]
pub async fn query_single_ts(
    object_id: Uuid,
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
    request_body: Body,
) -> Result<impl warp::Reply, warp::Rejection> {
    let SchemaMetadata {
        query_address,
        schema_type,
    } = routes::get_routing_info(schema_id, repository_id, cache, routing).await?;

    let Body { from, to, step } = request_body;

    let values = match schema_type {
        SchemaType::DocumentStorage => return Err(Error::InvalidSchemaType.into()),
        SchemaType::Timeseries => {
            let timeseries = rpc::query_service_ts::query_by_range(
                schema_id.to_string(),
                object_id.to_string(),
                from,
                to,
                step,
                query_address.clone(),
            )
            .await
            .map_err(Error::ClientError)?;

            timeseries.into_bytes()
        }
    };

    Ok(warp::reply::with_header(
        values,
        CONTENT_TYPE,
        APPLICATION_JSON,
    ))
}

#[tracing::instrument(skip(cache))]
pub async fn query_single_ds(
    object_id: Uuid,
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let SchemaMetadata {
        query_address,
        schema_type,
    } = routes::get_routing_info(schema_id, repository_id, cache, routing).await?;

    let values = match schema_type {
        SchemaType::Timeseries => return Err(Error::InvalidSchemaType.into()),
        SchemaType::DocumentStorage => {
            let mut values: HashMap<String, Vec<u8>> = rpc::query_service::query_multiple(
                vec![object_id.to_string()],
                query_address.clone(),
            )
            .await
            .map_err(Error::ClientError)?
            .map_ok(|object| (object.object_id, object.payload))
            .try_collect()
            .await
            .map_err(Error::ClientError)?;

            values
                .remove(&object_id.to_string())
                .ok_or(Error::SingleQueryMissingValue)?
        }
    };

    Ok(warp::reply::with_header(
        values,
        CONTENT_TYPE,
        APPLICATION_JSON,
    ))
}
