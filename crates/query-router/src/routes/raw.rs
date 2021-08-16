use std::collections::HashMap;

use rpc::{
    query_service,
    query_service_ts,
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
    raw_statement: String,
}

#[tracing::instrument(skip(cache))]
pub async fn query_raw(
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    request_body: Body,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let SchemaMetadata {
        query_address,
        schema_type,
    } = routes::get_routing_info(schema_id, repository_id, cache, routing).await?;

    let values = match &schema_type {
        SchemaType::DocumentStorage => {
            query_service::query_raw(request_body.raw_statement, query_address.clone())
                .await
                .map_err(Error::ClientError)
        }

        SchemaType::Timeseries => {
            query_service_ts::query_raw(request_body.raw_statement, query_address.clone())
                .await
                .map_err(Error::ClientError)
        }
    }?;

    Ok(warp::reply::with_header(
        values,
        CONTENT_TYPE,
        APPLICATION_JSON,
    ))
}
