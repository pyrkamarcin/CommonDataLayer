use std::collections::HashMap;
use std::num::NonZeroU8;
use std::time::Duration;

use anyhow::Result;
use cdl_api::types::view::NewRelation;
use cdl_dto::materialization::{
    Computation, EqualsComputation, FieldValueComputation, RawValueComputation,
};
use cdl_dto::materialization::{FieldDefinition, FieldType};
use cdl_rpc::schema_registry::types::SearchFor;
use tokio::time::sleep;
use uuid::Uuid;

use crate::{api::*, *};

#[tokio::test]
#[ignore = "todo"]
async fn should_compute_field_from_another_value_in_relationless_view() -> Result<()> {
    let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

    let mut fields = HashMap::new();
    fields.insert(
        "field_a".to_owned(),
        FieldDefinition::Simple {
            field_name: "FieldA".to_owned(),
            field_type: FieldType::Numeric,
        },
    );
    fields.insert(
        "field_b".to_owned(),
        FieldDefinition::Computed {
            computation: Computation::Equals(EqualsComputation {
                lhs: Box::new(Computation::RawValue(RawValueComputation {
                    value: serde_json::to_value("1")?.into(),
                })),
                rhs: Box::new(Computation::FieldValue(FieldValueComputation {
                    field_path: "FieldA".to_owned(),
                    schema_id: 0,
                })),
            }),
            field_type: FieldType::Json, // TODO: Boolean
        },
    );
    fields.insert(
        "field_c".to_owned(),
        FieldDefinition::Computed {
            computation: Computation::Equals(EqualsComputation {
                lhs: Box::new(Computation::RawValue(RawValueComputation {
                    value: serde_json::to_value("2")?.into(),
                })),
                rhs: Box::new(Computation::FieldValue(FieldValueComputation {
                    field_path: "FieldA".to_owned(),
                    schema_id: 0,
                })),
            }),
            field_type: FieldType::Json, // TODO: Boolean
        },
    );
    let object_id_a = Uuid::new_v4();

    let view = add_view(schema_a, "test", "", fields, None, &[], None).await?;
    insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view, &[schema_a]).await?;
    assert_eq!(view_data.rows.len(), 1);
    let row = view_data.rows.first().unwrap();
    let field_a = row.fields.get("field_a").unwrap().0.as_str().unwrap();
    let field_b = row.fields.get("field_b").unwrap().0.as_bool().unwrap();
    let field_c = row.fields.get("field_c").unwrap().0.as_bool().unwrap();
    assert_eq!(field_a, "1");
    assert!(field_b);
    assert!(!field_c);

    Ok(())
}

#[tokio::test]
async fn should_compute_field_when_more_than_one_objects_in_relations() -> Result<()> {
    let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let relation_id = add_relation(schema_a, schema_b).await?;

    let object_id_a = Uuid::new_v4();
    let object_id_b = Uuid::new_v4();
    insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
    insert_message(object_id_b, schema_b, "{}").await?;

    add_edges(relation_id, object_id_a, &[object_id_b]).await?;

    let mut fields = HashMap::new();
    fields.insert(
        "field_a".to_owned(),
        FieldDefinition::Simple {
            field_name: "FieldA".to_owned(),
            field_type: FieldType::Numeric,
        },
    );
    fields.insert(
        "field_b".to_owned(),
        FieldDefinition::Computed {
            computation: Computation::Equals(EqualsComputation {
                lhs: Box::new(Computation::RawValue(RawValueComputation {
                    value: serde_json::to_value(1)?.into(),
                })),
                rhs: Box::new(Computation::FieldValue(FieldValueComputation {
                    field_path: "FieldA".to_owned(),
                    schema_id: 0,
                })),
            }),
            field_type: FieldType::Json, // TODO: Boolean
        },
    );
    fields.insert(
        "field_c".to_owned(),
        FieldDefinition::Computed {
            computation: Computation::Equals(EqualsComputation {
                lhs: Box::new(Computation::RawValue(RawValueComputation {
                    value: serde_json::to_value(2)?.into(),
                })),
                rhs: Box::new(Computation::FieldValue(FieldValueComputation {
                    field_path: "FieldA".to_owned(),
                    schema_id: 0,
                })),
            }),
            field_type: FieldType::Json, // TODO: Boolean
        },
    );
    let view = add_view(
        schema_a,
        "test",
        "",
        fields,
        None,
        &[NewRelation {
            global_id: relation_id,
            local_id: NonZeroU8::new(1).unwrap(),
            relations: vec![],
            search_for: SearchFor::Children,
        }],
        None,
    )
    .await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
    assert_eq!(view_data.rows.len(), 1);
    let row = view_data.rows.first().unwrap();
    let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
    let field_b = row.fields.get("field_b").unwrap().0.as_bool().unwrap();
    let field_c = row.fields.get("field_c").unwrap().0.as_bool().unwrap();
    assert_eq!(field_a, 1);
    assert!(field_b);
    assert!(!field_c);

    Ok(())
}
