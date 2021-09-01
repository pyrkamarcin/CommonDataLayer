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
            definition,
            &[field_name.to_owned()],
        )?;
    }

    Ok(())
}

fn validate_data_inner(
    data: &Value,
    type_: &SchemaFieldDefinition,
    path: &[String],
) -> RegistryResult<()> {
    if data.is_null() {
        if type_.optional {
            return Ok(());
        } else {
            return Err(RegistryError::InvalidData(format!(
                "Value was unexpectedly null: Path `{}`",
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
                        validate_data_inner(field_data, field_definition, &field_path)?;
                    }
                    None => {
                        if !field_definition.optional {
                            return Err(RegistryError::InvalidData(format!(
                                "Value was null or missing: Path `{}`",
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
                validate_data_inner(item, item_type, &item_path)?;
            }
        }
    }

    Ok(())
}

pub fn wrong_type(expected_type: &str, path: &[String]) -> RegistryError {
    RegistryError::InvalidData(format!(
        "Expected data to be a {}: Path `{}`",
        expected_type,
        path.join(", ")
    ))
}

pub fn extend_path(base_path: &[String], segment: String) -> Vec<String> {
    base_path
        .iter()
        .cloned()
        .chain(Some(segment).into_iter())
        .collect()
}

#[cfg(test)]
mod tests {
    use ::types::schemas::{SchemaFieldDefinition, SchemaFieldType};
    use maplit::hashmap;
    use rpc::schema_registry::types::ScalarType;
    use serde_json::json;

    use super::*;

    fn schema_required() -> HashMap<String, SchemaFieldDefinition> {
        hashmap! {
            "field A".to_owned() => SchemaFieldDefinition {
                optional: false,
                field_type: SchemaFieldType::Scalar(ScalarType::Integer),
            }
        }
    }

    fn schema_optional() -> HashMap<String, SchemaFieldDefinition> {
        hashmap! {
            "foo".to_owned() => SchemaFieldDefinition {
                optional: true,
                field_type: SchemaFieldType::Object(hashmap! {
                    "bar".to_owned() => SchemaFieldDefinition {
                        optional: false,
                        field_type: SchemaFieldType::Scalar(ScalarType::Bool),
                    }
                }),
            }
        }
    }

    fn schema_array(allow_null_elements: bool) -> HashMap<String, SchemaFieldDefinition> {
        hashmap! {
            "foo".to_owned() => SchemaFieldDefinition {
                optional: true,
                field_type: SchemaFieldType::Array(Box::new(SchemaFieldDefinition {
                    optional: allow_null_elements,
                    field_type: SchemaFieldType::Scalar(ScalarType::String),
                })),
            }
        }
    }

    #[test]
    fn test_valid_data() {
        let data = json!({
            "field A": 1,
        });

        assert!(validate_data(&data, &schema_required()).is_ok());
    }

    #[test]
    fn test_invalid_data() {
        let data = json!({
            "field B": 1,
        });

        assert_eq!(
            validate_data(&data, &schema_required())
                .unwrap_err()
                .to_string(),
            "Input data does not match schema: Value was unexpectedly null: Path `field A`"
                .to_owned()
        );
    }

    #[test]
    fn test_data_wrong_type() {
        let data = json!({
            "field A": 1.5,
        });

        assert_eq!(
            validate_data(&data, &schema_required())
                .unwrap_err()
                .to_string(),
            "Input data does not match schema: Expected data to be a integer: Path `field A`"
                .to_owned()
        );
    }

    #[test]
    fn test_data_optional_with_missing_field() {
        let data = json!({});

        assert!(validate_data(&data, &schema_optional()).is_ok());
    }

    #[test]
    fn test_data_optional_with_null_field() {
        let data = json!({});

        assert!(validate_data(&data, &schema_optional()).is_ok());
    }

    #[test]
    fn test_array_no_null_elements_allowed() {
        let data = json!({ "foo": [Some("a"), Some("b"), Option::<&str>::None, Some("d")] });

        assert_eq!(
            validate_data(&data, &schema_array(false))
                .unwrap_err()
                .to_string(),
            "Input data does not match schema: Value was unexpectedly null: Path `foo, 2`"
                .to_owned()
        );
    }

    #[test]
    fn test_array_some_null_elements_allowed() {
        let data = json!({ "foo": [Some("a"), Some("b"), Option::<&str>::None, Some("d")] });

        assert!(validate_data(&data, &schema_array(true)).is_ok());
    }
}
