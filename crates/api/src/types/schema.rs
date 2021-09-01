use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use ::types::schemas::SchemaFieldDefinition;
use async_graphql::{FieldResult, InputObject, Json, SimpleObject};
use rpc::schema_registry::types::SchemaType;
use uuid::Uuid;

use crate::types::view::View;

/// Schema is the format in which data is to be sent to the Common Data Layer.
#[derive(Debug, SimpleObject)]
pub struct FullSchema {
    /// Random UUID assigned on creation.
    pub id: Uuid,
    /// The name is not required to be unique among all schemas (as `id` is the identifier).
    pub name: String,
    /// Message queue insert_destination to which data is inserted by data-router.
    pub insert_destination: String,
    /// Address of the query service responsible for retrieving data from DB.
    pub query_address: String,
    /// Whether this schema represents documents or timeseries data.
    pub schema_type: SchemaType,
    /// The format of data stored under this schema.
    pub definition: Json<HashMap<String, SchemaFieldDefinition>>,
    /// All views belonging to this schema.
    pub views: Vec<View>,
}

impl FullSchema {
    pub fn from_rpc(schema: rpc::schema_registry::FullSchema) -> FieldResult<Self> {
        let schema_type: rpc::schema_registry::types::SchemaType = schema.schema_type.try_into()?;

        Ok(FullSchema {
            id: Uuid::parse_str(&schema.id)?,
            name: schema.name,
            insert_destination: schema.insert_destination,
            query_address: schema.query_address,
            schema_type,
            definition: Json(
                schema
                    .definition
                    .into_iter()
                    .map(|(field_name, field_definition)| {
                        Ok((
                            field_name,
                            SchemaFieldDefinition::try_from(field_definition)?,
                        ))
                    })
                    .collect::<FieldResult<HashMap<_, _>>>()?,
            ),
            views: schema
                .views
                .into_iter()
                .map(View::from_rpc)
                .collect::<FieldResult<Vec<_>>>()?,
        })
    }
}

/// Input object which creates new schema and new definition. Each schema has to
/// contain at least one definition, which can be later overriden.
#[derive(Debug, InputObject)]
pub struct NewSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: String,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: String,
    /// Destination to which data is inserted by data-router.
    pub insert_destination: String,
    /// Definition is stored as a JSON value and therefore needs to be valid JSON.
    pub definition: Json<HashMap<String, SchemaFieldDefinition>>,
    /// Whether the schema stores documents or timeseries data.
    #[graphql(name = "type")]
    pub schema_type: SchemaType,
}

impl NewSchema {
    pub fn into_rpc(self) -> FieldResult<rpc::schema_registry::NewSchema> {
        Ok(rpc::schema_registry::NewSchema {
            name: self.name,
            schema_type: self.schema_type.into(),
            insert_destination: self.insert_destination,
            query_address: self.query_address,
            definition: self
                .definition
                .0
                .into_iter()
                .map(|(field_name, field_definition)| {
                    Ok((field_name, field_definition.try_into()?))
                })
                .collect::<FieldResult<HashMap<_, _>>>()?,
        })
    }
}

/// Input object which updates fields in schema. All fields are optional,
/// therefore one may update only `topic` or `queryAddress` or all of them.
#[derive(Debug, InputObject)]
pub struct UpdateSchema {
    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    pub name: Option<String>,
    /// Address of the query service responsible for retrieving data from DB
    pub query_address: Option<String>,
    /// Destination to which data is inserted by data-router.
    pub insert_destination: Option<String>,
    /// Whether the schema stores documents or timeseries data.
    #[graphql(name = "type")]
    pub schema_type: Option<SchemaType>,
    /// Definition of the data stored under this schema.
    pub definition: Option<Json<HashMap<String, SchemaFieldDefinition>>>,
}

impl UpdateSchema {
    pub fn into_rpc(self, id: Uuid) -> anyhow::Result<rpc::schema_registry::SchemaUpdate> {
        let (update_definition, definition) = if let Some(def) = self.definition {
            (
                true,
                def.0.into_iter()
                    .map(|(field_name, field_definition)| {
                        Ok((field_name, field_definition.try_into()?))
                    })
                    .collect::<anyhow::Result<HashMap<String, rpc::schema_registry::SchemaFieldDefinition>>>()?,
            )
        } else {
            (false, HashMap::new())
        };

        Ok(rpc::schema_registry::SchemaUpdate {
            id: id.to_string(),
            name: self.name,
            insert_destination: self.insert_destination,
            query_address: self.query_address,
            schema_type: self.schema_type.map(Into::into),
            update_definition,
            definition,
        })
    }
}
