use std::{convert::TryInto, path::PathBuf};

use rpc::schema_registry::{
    types::SchemaType,
    Empty,
    Id,
    NewSchema,
    SchemaUpdate,
    ValueToValidate,
};
use serde_json::Value;
use uuid::Uuid;

use crate::utils::*;

pub async fn get_schema_definition(schema_id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let response = client
        .get_schema(Id {
            id: schema_id.to_string(),
        })
        .await?;

    println!(
        "{:#}",
        serde_json::from_slice::<Value>(&response.into_inner().definition)?
    );

    Ok(())
}

pub async fn add_schema(
    schema_name: String,
    insert_destination: String,
    query_address: String,
    file: Option<PathBuf>,
    schema_type: SchemaType,
    registry_addr: String,
) -> anyhow::Result<()> {
    let definition = read_json(file)?;

    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let response = client
        .add_schema(NewSchema {
            name: schema_name.clone(),
            query_address,
            insert_destination,
            schema_type: schema_type.into(),
            definition: serde_json::to_vec(&definition)?,
        })
        .await?;

    eprintln!(
        "Successfully added schema \"{}\" to the schema registry.",
        schema_name
    );
    eprintln!("The following UUID was assigned:");
    println!("{}", response.into_inner().id);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn update_schema(
    id: Uuid,
    name: Option<String>,
    insert_destination: Option<String>,
    query_address: Option<String>,
    schema_type: Option<SchemaType>,
    update_definition: bool,
    file: Option<PathBuf>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    client
        .update_schema(SchemaUpdate {
            id: id.to_string(),
            name,
            insert_destination,
            query_address,
            schema_type: schema_type.map(|t| t.into()),
            definition: if update_definition {
                Some(serde_json::to_vec(&read_json(file)?)?)
            } else {
                None
            },
        })
        .await?;

    Ok(())
}

pub async fn get_schema_names(registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let mut schemas = client.get_all_schemas(Empty {}).await?.into_inner().schemas;

    if schemas.is_empty() {
        anyhow::bail!("No schemas exist yet in the schema registry.");
    }

    schemas.sort_by_key(|schema| schema.name.clone());
    for schema in schemas {
        println!("ID: {}, Name: {}", schema.id, schema.name);
    }

    Ok(())
}

pub async fn get_schema_metadata(id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let metadata = client
        .get_schema(Id { id: id.to_string() })
        .await?
        .into_inner();
    let schema_type: SchemaType = metadata.schema_type.try_into()?;

    println!("Name: {}", metadata.name);
    println!("Topic or Queue: {}", metadata.insert_destination);
    println!("Query Address: {}", metadata.query_address);
    println!("Type: {}", schema_type);

    Ok(())
}

pub async fn validate_value(
    schema_id: Uuid,
    file: Option<PathBuf>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let value = read_json(file)?;

    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let errors = client
        .validate_value(ValueToValidate {
            schema_id: schema_id.to_string(),
            value: serde_json::to_vec(&value)?,
        })
        .await?
        .into_inner()
        .errors;

    if !errors.is_empty() {
        anyhow::bail!("The value is not valid. {}", errors.join(", "));
    }

    println!("The value is valid.");
    Ok(())
}
