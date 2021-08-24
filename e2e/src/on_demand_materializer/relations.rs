use std::{collections::HashMap, num::NonZeroU8, time::Duration};

use anyhow::Result;
use cdl_api::types::view::NewRelation;
use cdl_dto::materialization::{
    Computation,
    EqualsFilter,
    FieldDefinition,
    FieldType,
    FieldValueComputation,
    Filter,
    FilterValue,
    RawValueFilter,
    SchemaFieldFilter,
    SimpleFilter,
    SimpleFilterKind,
};
use cdl_rpc::common::types::SearchFor;
use tokio::time::sleep;
use uuid::Uuid;

use crate::{api::*, *};

#[tokio::test]
async fn should_return_no_results_when_one_of_related_objects_does_not_exist() -> Result<()> {
    let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let relation_id = add_relation(schema_a, schema_b).await?;

    let view = add_view(
        schema_a,
        "test",
        "",
        Default::default(),
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

    let object_id_a = Uuid::new_v4();
    let object_id_b = Uuid::new_v4();
    insert_message(object_id_a, schema_a, "{}").await?;
    add_edges(relation_id, object_id_a, &[object_id_b]).await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
    assert_eq!(view_data.rows.len(), 0);

    Ok(())
}

#[tokio::test]
async fn should_return_no_results_when_edge_was_not_added() -> Result<()> {
    let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let relation_id = add_relation(schema_a, schema_b).await?;

    let view = add_view(
        schema_a,
        "test",
        "",
        Default::default(),
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

    let object_id_a = Uuid::new_v4();
    let object_id_b = Uuid::new_v4();
    insert_message(object_id_a, schema_a, "{}").await?;
    insert_message(object_id_b, schema_b, "{}").await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
    assert_eq!(view_data.rows.len(), 0);

    Ok(())
}

#[tokio::test]
async fn should_apply_inner_join_strategy() -> Result<()> {
    let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let relation_id = add_relation(schema_a, schema_b).await?;

    let view = add_view(
        schema_a,
        "test",
        "",
        Default::default(),
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

    let object_id_a = Uuid::new_v4();
    let object_id_b = Uuid::new_v4();
    insert_message(object_id_a, schema_a, "{}").await?;
    insert_message(object_id_b, schema_b, "{}").await?;
    add_edges(relation_id, object_id_a, &[object_id_b]).await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
    assert_eq!(view_data.rows.len(), 1);

    Ok(())
}

#[tokio::test]
async fn should_join_objects_from_parent_side() -> Result<()> {
    let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let relation_id = add_relation(schema_a, schema_a).await?;

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
            computation: Computation::FieldValue(FieldValueComputation {
                schema_id: 1,
                field_path: "FieldA".to_owned(),
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
        Some(Filter::SimpleFilter(SimpleFilter {
            filter: SimpleFilterKind::Equals(EqualsFilter {
                lhs: FilterValue::SchemaField(SchemaFieldFilter {
                    field_path: "FieldA".to_owned(),
                    schema_id: 0,
                }),
                rhs: FilterValue::RawValue(RawValueFilter {
                    value: serde_json::to_value(1)?.into(),
                }),
            }),
        })),
    )
    .await?;

    let object_id_a = Uuid::new_v4();
    let object_id_b = Uuid::new_v4();
    insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
    insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;
    add_edges(relation_id, object_id_a, &[object_id_b]).await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view, &[schema_a]).await?;
    assert_eq!(view_data.rows.len(), 1);

    let row = view_data.rows.first().unwrap();
    let field_a = row.fields.get("field_a").unwrap().0.as_i64().unwrap();
    let field_b = row.fields.get("field_b").unwrap().0.as_i64().unwrap();
    assert_eq!(field_a, 1);
    assert_eq!(field_b, 2);

    Ok(())
}

#[tokio::test]
async fn should_join_objects_from_child_side() -> Result<()> {
    let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
    let relation_id = add_relation(schema_a, schema_a).await?;

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
            computation: Computation::FieldValue(FieldValueComputation {
                schema_id: 1,
                field_path: "FieldA".to_owned(),
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
            search_for: SearchFor::Parents,
        }],
        Some(Filter::SimpleFilter(SimpleFilter {
            filter: SimpleFilterKind::Equals(EqualsFilter {
                lhs: FilterValue::SchemaField(SchemaFieldFilter {
                    field_path: "FieldA".to_owned(),
                    schema_id: 0,
                }),
                rhs: FilterValue::RawValue(RawValueFilter {
                    value: serde_json::to_value(2)?.into(),
                }),
            }),
        })),
    )
    .await?;

    let object_id_a = Uuid::new_v4();
    let object_id_b = Uuid::new_v4();
    insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
    insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;
    add_edges(relation_id, object_id_a, &[object_id_b]).await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view, &[schema_a]).await?;
    assert_eq!(view_data.rows.len(), 1);

    let row = view_data.rows.first().unwrap();
    let field_a = row.fields.get("field_a").unwrap().0.as_i64().unwrap();
    let field_b = row.fields.get("field_b").unwrap().0.as_i64().unwrap();
    assert_eq!(field_a, 2);
    assert_eq!(field_b, 1);

    Ok(())
}
