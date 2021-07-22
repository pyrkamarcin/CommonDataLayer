mod on_standard_field {
    use std::collections::HashMap;
    use std::num::NonZeroU8;
    use std::time::Duration;

    use anyhow::Result;
    use cdl_api::types::view::NewRelation;
    use cdl_dto::materialization::{
        ComplexFilter, Computation, EqualsFilter, FieldValueComputation, Filter, FilterValue,
        RawValueFilter, SchemaFieldFilter, SimpleFilter, SimpleFilterKind,
    };
    use cdl_dto::materialization::{ComputedFilter, FieldDefinition, FieldType, ViewPathFilter};
    use cdl_rpc::schema_registry::types::{LogicOperator, SearchFor};
    use tokio::time::sleep;
    use uuid::Uuid;

    use crate::{api::*, *};

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_apply_simple_filter_for_request_with_empty_relations() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
        insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        let view = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[],
            Some(Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter {
                    lhs: FilterValue::SchemaField(SchemaFieldFilter {
                        field_path: "FieldA".to_owned(),
                        schema_id: 0,
                    }),
                    rhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(&1)?.into(),
                    }),
                }),
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a]).await?;
        assert_eq!(view_data.rows.len(), 1);
        let row = view_data.rows.first().unwrap();
        let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
        assert_eq!(field_a, 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_properly_merge_multiple_filters_using_and_operator() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        let object_id_c = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1,"FieldB":1}"#).await?;
        insert_message(object_id_b, schema_a, r#"{"FieldA":2,"FieldB":1}"#).await?;
        insert_message(object_id_c, schema_a, r#"{"FieldA":2,"FieldB":2}"#).await?;

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
            FieldDefinition::Simple {
                field_name: "FieldB".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        let view = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[],
            Some(Filter::ComplexFilter(ComplexFilter {
                operator: LogicOperator::And,
                operands: vec![
                    Filter::SimpleFilter(SimpleFilter {
                        filter: SimpleFilterKind::Equals(EqualsFilter {
                            lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                field_path: "FieldA".to_owned(),
                                schema_id: 0,
                            }),
                            rhs: FilterValue::RawValue(RawValueFilter {
                                value: serde_json::to_value(&1)?.into(),
                            }),
                        }),
                    }),
                    Filter::SimpleFilter(SimpleFilter {
                        filter: SimpleFilterKind::Equals(EqualsFilter {
                            lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                field_path: "FieldB".to_owned(),
                                schema_id: 0,
                            }),
                            rhs: FilterValue::RawValue(RawValueFilter {
                                value: serde_json::to_value(&2)?.into(),
                            }),
                        }),
                    }),
                ],
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a]).await?;
        assert_eq!(view_data.rows.len(), 1);
        let row = view_data.rows.first().unwrap();
        let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
        let field_b = row.fields.get("field_b").unwrap().0.as_u64().unwrap();
        assert_eq!(field_a, 1);
        assert_eq!(field_b, 2);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_properly_merge_multiple_filters_using_or_operator() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        let object_id_c = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
        insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;
        insert_message(object_id_c, schema_a, r#"{"FieldA":3}"#).await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        let view = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[],
            Some(Filter::ComplexFilter(ComplexFilter {
                operator: LogicOperator::Or,
                operands: vec![
                    Filter::SimpleFilter(SimpleFilter {
                        filter: SimpleFilterKind::Equals(EqualsFilter {
                            lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                field_path: "FieldA".to_owned(),
                                schema_id: 0,
                            }),
                            rhs: FilterValue::RawValue(RawValueFilter {
                                value: serde_json::to_value(&1)?.into(),
                            }),
                        }),
                    }),
                    Filter::SimpleFilter(SimpleFilter {
                        filter: SimpleFilterKind::Equals(EqualsFilter {
                            lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                field_path: "FieldA".to_owned(),
                                schema_id: 0,
                            }),
                            rhs: FilterValue::RawValue(RawValueFilter {
                                value: serde_json::to_value(&2)?.into(),
                            }),
                        }),
                    }),
                ],
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a]).await?;
        assert_eq!(view_data.rows.len(), 2);
        let mut values = view_data
            .rows
            .into_iter()
            .map(|row| row.fields.get("field_a").unwrap().0.as_u64().unwrap())
            .collect::<Vec<_>>();
        values.sort_unstable();

        assert_eq!(values.get(0).unwrap(), &1);
        assert_eq!(values.get(1).unwrap(), &2);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_properly_merge_multiple_filters_complex() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        let object_id_c = Uuid::new_v4();
        let object_id_d = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1,"FieldB":1}"#).await?;
        insert_message(object_id_b, schema_a, r#"{"FieldA":2,"FieldB":1}"#).await?;
        insert_message(object_id_c, schema_a, r#"{"FieldA":1,"FieldB":2}"#).await?;
        insert_message(object_id_d, schema_a, r#"{"FieldA":2,"FieldB":2}"#).await?;

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
            FieldDefinition::Simple {
                field_name: "FieldB".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        let view = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[],
            Some(Filter::ComplexFilter(ComplexFilter {
                operator: LogicOperator::Or,
                operands: vec![
                    Filter::ComplexFilter(ComplexFilter {
                        operator: LogicOperator::And,
                        operands: vec![
                            Filter::SimpleFilter(SimpleFilter {
                                filter: SimpleFilterKind::Equals(EqualsFilter {
                                    lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                        field_path: "FieldA".to_owned(),
                                        schema_id: 0,
                                    }),
                                    rhs: FilterValue::RawValue(RawValueFilter {
                                        value: serde_json::to_value(&1)?.into(),
                                    }),
                                }),
                            }),
                            Filter::SimpleFilter(SimpleFilter {
                                filter: SimpleFilterKind::Equals(EqualsFilter {
                                    lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                        field_path: "FieldB".to_owned(),
                                        schema_id: 0,
                                    }),
                                    rhs: FilterValue::RawValue(RawValueFilter {
                                        value: serde_json::to_value(&1)?.into(),
                                    }),
                                }),
                            }),
                        ],
                    }),
                    Filter::ComplexFilter(ComplexFilter {
                        operator: LogicOperator::And,
                        operands: vec![
                            Filter::SimpleFilter(SimpleFilter {
                                filter: SimpleFilterKind::Equals(EqualsFilter {
                                    lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                        field_path: "FieldA".to_owned(),
                                        schema_id: 0,
                                    }),
                                    rhs: FilterValue::RawValue(RawValueFilter {
                                        value: serde_json::to_value(&2)?.into(),
                                    }),
                                }),
                            }),
                            Filter::SimpleFilter(SimpleFilter {
                                filter: SimpleFilterKind::Equals(EqualsFilter {
                                    lhs: FilterValue::SchemaField(SchemaFieldFilter {
                                        field_path: "FieldB".to_owned(),
                                        schema_id: 0,
                                    }),
                                    rhs: FilterValue::RawValue(RawValueFilter {
                                        value: serde_json::to_value(&2)?.into(),
                                    }),
                                }),
                            }),
                        ],
                    }),
                ],
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a]).await?;

        let mut values = view_data
            .rows
            .into_iter()
            .map(|row| {
                (
                    row.fields.get("field_a").unwrap().0.as_u64().unwrap(),
                    row.fields.get("field_b").unwrap().0.as_u64().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        values.sort_unstable();

        assert_eq!(values.len(), 2);
        assert_eq!(values.get(0).unwrap().0, 1);
        assert_eq!(values.get(0).unwrap().1, 1);
        assert_eq!(values.get(1).unwrap().0, 2);
        assert_eq!(values.get(1).unwrap().1, 2);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_apply_filters_to_subrelations() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let relation_id = add_relation(schema_a, schema_b).await?;

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
            FieldDefinition::SubObject {
                base: 1,
                fields: vec![(
                    "field_c".to_owned(),
                    FieldDefinition::Simple {
                        field_name: "FieldB".to_owned(),
                        field_type: FieldType::Numeric,
                    },
                )]
                .into_iter()
                .collect(),
            },
        );

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        let object_id_c = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
        insert_message(object_id_b, schema_b, r#"{"FieldB":1}"#).await?;
        insert_message(object_id_c, schema_b, r#"{"FieldB":2}"#).await?;
        add_edges(relation_id, object_id_a, &[object_id_b, object_id_c]).await?;

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
            Some(Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter {
                    lhs: FilterValue::SchemaField(SchemaFieldFilter {
                        field_path: "FieldB".to_owned(),
                        schema_id: 1,
                    }),
                    rhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(&1)?.into(),
                    }),
                }),
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
        assert_eq!(view_data.rows.len(), 1);

        let row = view_data.rows.first().unwrap();
        let field_b = row.fields.get("field_b").unwrap().0.clone();
        let field_c = (field_b["field_c"]).as_u64().unwrap();
        assert_eq!(field_c, 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_allow_filtering_using_field_not_materialized_in_view() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1, "FieldB":1}"#).await?;
        insert_message(object_id_b, schema_a, r#"{"FieldA":2, "FieldB":2}"#).await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        let view = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[],
            Some(Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter {
                    lhs: FilterValue::SchemaField(SchemaFieldFilter {
                        field_path: "FieldB".to_owned(),
                        schema_id: 0,
                    }),
                    rhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(&1)?.into(),
                    }),
                }),
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a]).await?;
        assert_eq!(view_data.rows.len(), 1);
        let row = view_data.rows.first().unwrap();
        let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
        assert_eq!(field_a, 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_allow_filtering_using_field_from_schema_not_materialized_in_view() -> Result<()>
    {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let schema_b = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;
        let relation_id = add_relation(schema_a, schema_b).await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::Numeric,
            },
        );

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        let object_id_c = Uuid::new_v4();
        let object_id_d = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
        insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;
        insert_message(object_id_c, schema_b, r#"{"FieldB":1}"#).await?;
        insert_message(object_id_d, schema_b, r#"{"FieldB":2}"#).await?;
        add_edges(relation_id, object_id_a, &[object_id_c]).await?;
        add_edges(relation_id, object_id_b, &[object_id_d]).await?;

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
            Some(Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter {
                    lhs: FilterValue::SchemaField(SchemaFieldFilter {
                        field_path: "FieldB".to_owned(),
                        schema_id: 1,
                    }),
                    rhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(&1)?.into(),
                    }),
                }),
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a, schema_b]).await?;
        assert_eq!(view_data.rows.len(), 1);

        let row = view_data.rows.first().unwrap();
        let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
        assert_eq!(field_a, 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_allow_filtering_using_fields_from_view() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1, "FieldB":1}"#).await?;
        insert_message(object_id_b, schema_a, r#"{"FieldA":1, "FieldB":2}"#).await?;

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
            FieldDefinition::Simple {
                field_name: "FieldB".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        let view = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[],
            Some(Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter {
                    lhs: FilterValue::ViewPath(ViewPathFilter {
                        field_path: "field_a".to_owned(),
                    }),
                    rhs: FilterValue::ViewPath(ViewPathFilter {
                        field_path: "field_b".to_owned(),
                    }),
                }),
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a]).await?;
        assert_eq!(view_data.rows.len(), 1);
        let row = view_data.rows.first().unwrap();
        let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
        let field_b = row.fields.get("field_b").unwrap().0.as_u64().unwrap();
        assert_eq!(field_a, 1);
        assert_eq!(field_b, 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_allow_filtering_using_raw_value() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let object_id_a = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1, "FieldB":1}"#).await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        let view_a = add_view(
            schema_a,
            "test",
            "",
            fields.clone(),
            None,
            &[],
            Some(Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter {
                    lhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(0)?.into(),
                    }),
                    rhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(1)?.into(),
                    }),
                }),
            })),
        )
        .await?;
        let view_b = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[],
            Some(Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter {
                    lhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(1)?.into(),
                    }),
                    rhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(1)?.into(),
                    }),
                }),
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data_a = materialize_view(view_a, &[schema_a]).await?;
        let view_data_b = materialize_view(view_b, &[schema_a]).await?;
        assert_eq!(view_data_a.rows.len(), 0);
        assert_eq!(view_data_b.rows.len(), 1);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "todo"]
    async fn should_allow_filtering_using_computed_field() -> Result<()> {
        let schema_a = add_schema("test", POSTGRES_QUERY_ADDR, POSTGRES_INSERT_DESTINATION).await?;

        let object_id_a = Uuid::new_v4();
        let object_id_b = Uuid::new_v4();
        insert_message(object_id_a, schema_a, r#"{"FieldA":1}"#).await?;
        insert_message(object_id_b, schema_a, r#"{"FieldA":2}"#).await?;

        let mut fields = HashMap::new();
        fields.insert(
            "field_a".to_owned(),
            FieldDefinition::Simple {
                field_name: "FieldA".to_owned(),
                field_type: FieldType::Numeric,
            },
        );
        let view = add_view(
            schema_a,
            "test",
            "",
            fields,
            None,
            &[],
            Some(Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter {
                    lhs: FilterValue::Computed(ComputedFilter {
                        computation: Computation::FieldValue(FieldValueComputation {
                            field_path: "FieldA".to_owned(),
                            schema_id: 0,
                        }),
                    }),
                    rhs: FilterValue::RawValue(RawValueFilter {
                        value: serde_json::to_value(1)?.into(),
                    }),
                }),
            })),
        )
        .await?;

        sleep(Duration::from_secs(1)).await; // async insert

        let view_data = materialize_view(view, &[schema_a]).await?;
        assert_eq!(view_data.rows.len(), 1);
        let row = view_data.rows.first().unwrap();
        let field_a = row.fields.get("field_a").unwrap().0.as_u64().unwrap();
        assert_eq!(field_a, 1);

        Ok(())
    }
}
