use async_graphql::{Json, SimpleObject};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, SimpleObject)]
pub struct Report {
    /// Application which generated the report
    pub application: String,
    /// Output plugin in command service
    pub output_plugin: Option<String>,
    /// Success/Failure
    pub description: String,
    /// Object id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_id: Option<Uuid>,
    /// JSON encoded payload
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Json<Value>>,
}
