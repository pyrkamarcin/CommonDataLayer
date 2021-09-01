use std::collections::HashMap;

use ::types::schemas::{SchemaFieldDefinition, SchemaFieldType};
use rpc::schema_registry::types::ScalarType;
use serde_json::Value;

use crate::error::{RegistryError, RegistryResult};

pub fn validate_data(
    data: &Value,
    schema_definition: &HashMap<String, SchemaFieldDefinition>,
) -> RegistryResult<()> {
    let fields = data.as_object().ok_or_else(|| {
        RegistryError::InvalidData("Data must be an object at the top level".to_owned())
    })?;

    for (field_name, definition) in schema_definition.iter() {
        validate_data_inner(
            fields.get(field_name).as_ref().unwrap_or(&&Value::Null),
            &definition,
            &vec![field_name.to_owned()],
        )?;
    }

    Ok(())
}

fn validate_data_inner(
    data: &Value,
    type_: &SchemaFieldDefinition,
    path: &Vec<String>,
) -> RegistryResult<()> {
    if data.is_null() {
        if type_.optional {
            return Ok(());
        } else {
            return Err(RegistryError::InvalidData(format!(
                "Value was unexpectedly null: path `{}`",
                path.join(", ")
            )));
        }
    }

    match &type_.field_type {
        SchemaFieldType::Scalar(scalar_type) => match scalar_type {
            ScalarType::Bool => {
                if !data.is_boolean() {
                    return Err(wrong_type("boolean", path));
                }
            }
            ScalarType::String => {
                if !data.is_string() {
                    return Err(wrong_type("string", path));
                }
            }
            ScalarType::Decimal => {
                if !data.is_f64() {
                    return Err(wrong_type("decimal", path));
                }
            }
            ScalarType::Integer => {
                if !data.is_u64() && !data.is_i64() {
                    return Err(wrong_type("integer", path));
                }
            }
            ScalarType::Any => {}
        },
        SchemaFieldType::Object(fields) => {
            let data_fields = data.as_object().ok_or_else(|| wrong_type("object", path))?;

            for (field_name, field_definition) in fields.iter() {
                let field_path = extend_path(path, field_name.to_owned());
                match data_fields.get(field_name).filter(|f| !f.is_null()) {
                    Some(field_data) => {
                        validate_data_inner(field_data, &field_definition, &field_path)?;
                    }
                    None => {
                        if !field_definition.optional {
                            return Err(RegistryError::InvalidData(format!(
                                "Value was null or missing: path `{}`",
                                field_path.join(", ")
                            )));
                        }
                    }
                }
            }
        }
        SchemaFieldType::Array(item_type) => {
            let items = data.as_array().ok_or_else(|| wrong_type("object", path))?;

            for (index, item) in items.iter().enumerate() {
                let item_path = extend_path(path, index.to_string());
                validate_data_inner(item, &item_type, &item_path)?;
            }
        }
    }

    Ok(())
}

pub fn wrong_type(expected_type: &str, path: &Vec<String>) -> RegistryError {
    RegistryError::InvalidData(format!(
        "Expected data to be a {}: path `{}`",
        expected_type,
        path.join(", ")
    ))
}

pub fn extend_path(base_path: &Vec<String>, segment: String) -> Vec<String> {
    base_path
        .iter()
        .cloned()
        .chain(Some(segment).into_iter())
        .collect()
}
