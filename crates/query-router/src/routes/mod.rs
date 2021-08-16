use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use settings_utils::apps::RepositoryStaticRouting;
use uuid::Uuid;

use crate::{
    error::Error,
    schema::{SchemaCache, SchemaMetadata},
};

pub mod by_schema;
pub mod multiple;
pub mod raw;
pub mod single;

const APPLICATION_JSON: &str = "application/json";

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Body {
    Range {
        from: String,
        to: String,
        step: String,
    },
    Raw {
        raw_statement: String,
    },
    Empty,
}

fn byte_map_to_json_map(map: HashMap<String, Vec<u8>>) -> Result<HashMap<String, Value>, Error> {
    map.into_iter()
        .map(|(object_id, value)| {
            Ok((
                object_id,
                serde_json::from_slice(&value).map_err(Error::JsonError)?,
            ))
        })
        .collect()
}

async fn get_routing_info(
    schema_id: Uuid,
    repository_id: Option<String>,
    cache: Arc<SchemaCache>,
    routing: Arc<HashMap<String, RepositoryStaticRouting>>,
) -> Result<SchemaMetadata, Error> {
    let metadata = if let Some(repository_id) = repository_id {
        let entry = routing.get(&repository_id);
        if let Some(routing) = entry {
            SchemaMetadata {
                query_address: routing.query_address.clone(),
                schema_type: routing.repository_type.into(),
            }
        } else {
            return Err(Error::InvalidRepository(repository_id));
        }
    } else {
        cache
            .get(schema_id)
            .await
            .map_err(Error::SchemaFetchError)?
    };

    Ok(metadata)
}
