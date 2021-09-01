use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
};

use rpc::schema_registry::{
    schema_field_type,
    types::ScalarType,
    SchemaFieldDefinition as SchemaFieldDefinitionRpc,
    SchemaFieldType as SchemaFieldTypeRpc,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SchemaFieldDefinition {
    pub field_type: SchemaFieldType,
    pub optional: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SchemaFieldType {
    Scalar(ScalarType),
    Object(HashMap<String, SchemaFieldDefinition>),
    Array(Box<SchemaFieldDefinition>),
}

impl TryFrom<SchemaFieldDefinitionRpc> for SchemaFieldDefinition {
    type Error = anyhow::Error;

    fn try_from(definition: SchemaFieldDefinitionRpc) -> Result<Self, Self::Error> {
        let old_type = definition.field_type;
        let mapped_type = match old_type.field_type {
            0 => SchemaFieldType::Scalar(
                old_type
                    .scalar_type
                    .ok_or_else(|| anyhow::anyhow!("Missing scalar type"))?
                    .try_into()?,
            ),
            1 => SchemaFieldType::Object(
                old_type
                    .field_types
                    .into_iter()
                    .map(|(f_name, f_type)| Ok((f_name, SchemaFieldDefinition::try_from(f_type)?)))
                    .collect::<anyhow::Result<_>>()?,
            ),
            2 => SchemaFieldType::Array(Box::new(
                (*old_type
                    .item_type
                    .ok_or_else(|| anyhow::anyhow!("Missing item type"))?)
                .try_into()?,
            )),
            _ => anyhow::bail!("Invalid field type"),
        };

        Ok(SchemaFieldDefinition {
            optional: definition.optional,
            field_type: mapped_type,
        })
    }
}

impl TryFrom<SchemaFieldDefinition> for SchemaFieldDefinitionRpc {
    type Error = anyhow::Error;

    fn try_from(definition: SchemaFieldDefinition) -> Result<Self, Self::Error> {
        let mapped_type = match definition.field_type {
            SchemaFieldType::Scalar(scalar_type) => SchemaFieldTypeRpc {
                field_type: schema_field_type::Type::Scalar.into(),
                scalar_type: Some(scalar_type.into()),
                item_type: None,
                field_types: HashMap::new(),
            },
            SchemaFieldType::Object(field_types) => SchemaFieldTypeRpc {
                field_type: schema_field_type::Type::Object.into(),
                scalar_type: None,
                item_type: None,
                field_types: field_types
                    .into_iter()
                    .map(|(field_name, field_definition)| {
                        Ok((
                            field_name,
                            SchemaFieldDefinitionRpc::try_from(field_definition)?,
                        ))
                    })
                    .collect::<anyhow::Result<_>>()?,
            },
            SchemaFieldType::Array(item_type) => SchemaFieldTypeRpc {
                field_type: schema_field_type::Type::Array.into(),
                scalar_type: None,
                item_type: Some(Box::new(SchemaFieldDefinitionRpc::try_from(*item_type)?)),
                field_types: HashMap::new(),
            },
        };

        Ok(SchemaFieldDefinitionRpc {
            optional: definition.optional,
            field_type: Box::new(mapped_type),
        })
    }
}
