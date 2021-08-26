use rpc::schema_registry::types::SchemaType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::types::view::FullView;

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

pub struct ObjectFieldDefinition {
    pub field_type: ScalarType,
    pub optional: bool,
}

pub enum ScalarType {
    Bool(bool),
    String(String),
    Integer(i64),
    Decimal(f64),
    Any(Value),
}

pub enum SchemaDefinition {
    Scalar {
        scalar_type: ScalarType,
        optional: bool,
    },
    Object {
        fields: HashMap<String, ObjectFieldDefinition>,
    },
    Array {
        item_type: SchemaChildType,
        optional: bool,
    }
}
