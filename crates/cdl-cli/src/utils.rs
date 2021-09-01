use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    path::PathBuf,
};

use ::types::schemas::SchemaFieldDefinition;
use anyhow::Context;
use rpc::schema_registry::SchemaFieldDefinition as SchemaFieldDefinitionRpc;
use serde::de::DeserializeOwned;

pub fn read_json<T: DeserializeOwned>(file: Option<PathBuf>) -> anyhow::Result<T> {
    if let Some(file_name) = file {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(file_name)
            .context("Couldn't open file")?;
        serde_json::from_reader(file).context("File did not contain valid JSON")
    } else {
        serde_json::from_reader(std::io::stdin()).context("File did not contain valid JSON")
    }
}

pub fn convert_definition_from_rpc(
    definition: HashMap<String, SchemaFieldDefinitionRpc>,
) -> anyhow::Result<HashMap<String, SchemaFieldDefinition>> {
    definition
        .into_iter()
        .map(|(field_name, field_definition)| {
            Ok((
                field_name,
                SchemaFieldDefinition::try_from(field_definition)
                    .map_err(|err| anyhow::anyhow!("Invalid definition: {}", err))?,
            ))
        })
        .collect::<anyhow::Result<_>>()
}

pub fn convert_definition_to_rpc(
    definition: HashMap<String, SchemaFieldDefinition>,
) -> anyhow::Result<HashMap<String, SchemaFieldDefinitionRpc>> {
    definition
        .into_iter()
        .map(|(field_name, field_definition)| {
            Ok((
                field_name,
                field_definition
                    .try_into()
                    .map_err(|err| anyhow::anyhow!("Invalid definition: {}", err))?,
            ))
        })
        .collect::<anyhow::Result<_>>()
}
