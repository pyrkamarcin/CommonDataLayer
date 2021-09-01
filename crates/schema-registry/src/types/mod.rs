use std::collections::HashMap;

use ::types::schemas::SchemaFieldDefinition;
use rpc::schema_registry::types::SchemaType;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

use self::view::View;

pub mod schema;
pub mod view;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DbExport {
    pub schemas: Vec<ExportSchema>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExportSchema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definition: Json<HashMap<String, SchemaFieldDefinition>>,
    pub views: Vec<View>,
}

#[cfg(test)]
mod tests {
    use ::types::schemas::{SchemaFieldDefinition, SchemaFieldType};
    use cdl_dto::materialization::{FieldDefinition, FieldType};
    use maplit::hashmap;
    use rpc::schema_registry::types::{ScalarType, SchemaType};
    use serde_json::json;
    use sqlx::types::Json;

    use super::*;

    #[test]
    fn test_deserialize_db_export() {
        let input = include_str!("test-db-export.json");

        let db_export: DbExport = serde_json::from_str(input).expect("Deserialized import file");
        diff_assert::assert_dbg!(
            DbExport {
                schemas: vec![
                    ExportSchema {
                        id: "2cfad3c7-411a-11eb-8000-000000000000".parse().unwrap(),
                        name: "new schema".into(),
                        insert_destination: "cdl.document.1.data".into(),
                        query_address: "http://localhost:50201".into(),
                        schema_type: SchemaType::DocumentStorage,
                        definition: Json(hashmap! {
                            "foo".into() => SchemaFieldDefinition {
                                optional: true,
                                field_type: SchemaFieldType::Scalar(ScalarType::String),
                            }
                        }),
                        views: vec![View {
                            id: "ec8cc976-412b-11eb-8000-000000000000".parse().unwrap(),
                            name: "new view".into(),
                            materializer_address: "http://localhost:50203".into(),
                            materializer_options: json!({
                                "table": "MATERIALIZED_VIEW"
                            }),
                            fields: Json(hashmap! {
                                "foo".into() => FieldDefinition::Simple {
                                    field_name: "a".into(),
                                    field_type: FieldType::String
                                }
                            }),
                            relations: Json(vec![]),
                            filters: Json(None)
                        }]
                    },
                    ExportSchema {
                        id: "a5e0c7e2-412c-11eb-8000-000000000000".parse().unwrap(),
                        name: "second schema".into(),
                        insert_destination: "cdl.document.1.data".into(),
                        query_address: "http://localhost:50201".into(),
                        schema_type: SchemaType::DocumentStorage,
                        definition: Json(hashmap! {
                            "foo".into() => SchemaFieldDefinition {
                                optional: true,
                                field_type: SchemaFieldType::Scalar(ScalarType::String),
                            }
                        }),
                        views: vec![]
                    }
                ]
            },
            db_export
        )
    }
}
