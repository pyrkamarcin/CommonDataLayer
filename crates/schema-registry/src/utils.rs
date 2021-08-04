use jsonschema::JSONSchema;
use serde_json::Value;
use uuid::Uuid;

use crate::db::SchemaRegistryDb;
use crate::error::{RegistryError, RegistryResult};

pub async fn build_full_schema(schema: &mut Value, conn: &SchemaRegistryDb) -> RegistryResult<()> {
    if let Some(defs) = schema
        .get_mut("definitions")
        .and_then(|val| val.as_object_mut())
    {
        let schemas_to_retrieve: Vec<(String, Uuid)> = defs
            .iter()
            .filter_map(|(key, value)| {
                value
                    .as_str()
                    .and_then(|val| Uuid::parse_str(val).ok())
                    .map(|id| (key.clone(), id))
            })
            .collect();

        for (key, id) in schemas_to_retrieve {
            let definition = conn.get_schema(id).await?.definition;
            defs[&key] = definition;
        }
    }

    JSONSchema::compile(schema).map_err(|err| RegistryError::InvalidJsonSchema(err.to_string()))?;

    Ok(())
}
