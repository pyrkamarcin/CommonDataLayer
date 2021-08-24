use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use cdl_dto::materialization::{FieldDefinition, FieldType};
use tokio::time::sleep;
use uuid::Uuid;

use crate::{api::*, *};

#[tokio::test]
async fn should_generate_empty_result_set_for_view_without_objects() -> Result<()> {
    let schema_id = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

    let mut fields = HashMap::new();
    fields.insert(
        "field_a".to_owned(),
        FieldDefinition::Simple {
            field_name: "FieldA".to_owned(),
            field_type: FieldType::Numeric,
        },
    );

    let view_id = add_view(
        schema_id,
        "test",
        "",
        fields,
        None,
        Default::default(),
        None,
    )
    .await?; // TODO: Materializer_addr - should be optional if none view should not be automatically materialized(only on demand)

    let view_data = materialize_view(view_id, &[schema_id]).await?;
    assert!(view_data.rows.is_empty());
    Ok(())
}

#[tokio::test]
async fn should_generate_results() -> Result<()> {
    let schema_id = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

    let mut fields = HashMap::new();
    fields.insert(
        "field_a".to_owned(),
        FieldDefinition::Simple {
            field_name: "FieldA".to_owned(),
            field_type: FieldType::Numeric,
        },
    );

    let view_id = add_view(
        schema_id,
        "test",
        "",
        fields,
        None,
        Default::default(),
        None,
    )
    .await?;
    let object_id = Uuid::new_v4();
    insert_message(object_id, schema_id, r#"{"FieldA":1}"#).await?;

    sleep(Duration::from_secs(1)).await; // async insert

    let view_data = materialize_view(view_id, &[schema_id]).await?;
    assert_eq!(view_data.rows.len(), 1);
    assert!(view_data
        .rows
        .iter()
        .any(|x| x.object_ids.contains(&object_id)));
    Ok(())
}
