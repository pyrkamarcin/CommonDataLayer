use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::types::view::FullView;
use rpc::schema_registry::types::SchemaType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaUpdate {
    pub name: Option<String>,
    pub insert_destination: Option<String>,
    pub query_address: Option<String>,
    #[serde(rename = "type")]
    pub schema_type: Option<SchemaType>,
    pub definition: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FullSchema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definition: Value,
    pub views: Vec<FullView>,
}
