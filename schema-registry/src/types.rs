use crate::db::property;
use indradb::VertexProperties;
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub definition: Value,
    pub kafka_topic: String,
    pub query_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchemaVersion {
    pub version: Version,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub version: Version,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct View {
    pub name: String,
    pub jmespath: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedUuid {
    pub id: Uuid,
    pub version_req: VersionReq,
}

fn get_vertex_property_or<T: DeserializeOwned>(
    properties: &mut VertexProperties,
    name: &'static str,
) -> Option<T> {
    properties
        .props
        .drain_filter(|prop| prop.name == name)
        .next()
        .and_then(|prop| serde_json::from_value(prop.value).ok())
}

impl View {
    pub fn from_properties(mut properties: VertexProperties) -> Option<(Uuid, View)> {
        Some((
            properties.vertex.id,
            View {
                name: get_vertex_property_or(&mut properties, property::VIEW_NAME)?,
                jmespath: get_vertex_property_or(&mut properties, property::VIEW_EXPRESSION)?,
            },
        ))
    }
}

impl VersionedUuid {
    pub fn new(id: Uuid, version_req: VersionReq) -> Self {
        Self { id, version_req }
    }

    pub fn exact(id: Uuid, version: Version) -> Self {
        Self {
            id,
            version_req: VersionReq::exact(&version),
        }
    }

    pub fn any(id: Uuid) -> Self {
        Self {
            id,
            version_req: VersionReq::any(),
        }
    }
}
