use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU8,
};

use anyhow::{Context, Result};
use cdl_dto::{
    edges::RelationTree,
    materialization::{
        ComplexFilter,
        Computation,
        ComputedFilter,
        EqualsComputation,
        EqualsFilter,
        FieldDefinition,
        FieldValueComputation,
        Filter,
        FilterValue,
        FullView,
        RawValueComputation,
        RawValueFilter,
        SchemaFieldFilter,
        SimpleFilter,
        SimpleFilterKind,
        ViewPathFilter,
    },
};
use itertools::Itertools;
use uuid::Uuid;

use super::UnfinishedRow;
use crate::{
    sources::{ComputationSource, FieldDefinitionSource, FilterSource, FilterValueSource},
    utils::flat_relation,
    view_plan::UnfinishedRowVariant,
    ObjectIdPair,
};

#[derive(Debug)]
pub struct ViewPlanBuilder<'a> {
    pub view: &'a FullView,
    pub relations: HashMap<Uuid, NonZeroU8>,
    pub relation_map: HashMap<Uuid, (Uuid, Uuid)>,
}

impl<'a> ViewPlanBuilder<'a> {
    pub fn new(view: &'a FullView, relation_map: HashMap<Uuid, (Uuid, Uuid)>) -> Self {
        let relations = Self::relations(view);

        Self {
            view,
            relations,
            relation_map,
        }
    }

    pub fn build_single_row(&self, root_object: ObjectIdPair) -> Result<UnfinishedRow> {
        let variant = UnfinishedRowVariant {
            root_object,
            objects: Default::default(),
        };

        let fields = self.build_fields(&variant, &self.view.fields)?;
        let filters = self
            .view
            .filters
            .as_ref()
            .map(|filters| self.build_filters(&variant, filters))
            .transpose()?;

        Ok(UnfinishedRow {
            missing: 1,
            objects: Default::default(),
            fields,
            filters,
            root_object,
            relation_order: vec![root_object],
        })
    }

    pub fn build_rows(
        &self,
        relation_tree: RelationTree,
    ) -> Result<Vec<(UnfinishedRow, HashSet<ObjectIdPair>)>> {
        relation_tree
            .rows
            .into_iter()
            .map(|row| {
                let variant = UnfinishedRowVariant::form_row(row, self.view, &self.relation_map);
                let fields = self.build_fields(&variant, &self.view.fields)?;
                let filters = self
                    .view
                    .filters
                    .as_ref()
                    .map(|filters| self.build_filters(&variant, filters))
                    .transpose()?;

                let mut set: HashSet<ObjectIdPair> =
                    variant.objects.iter().map(|(_, object)| *object).collect();

                set.insert(variant.root_object);

                let mut relation_order = vec![variant.root_object];
                for obj in variant.objects.iter().sorted().enumerate() {
                    debug_assert_eq!(NonZeroU8::new(obj.0 as u8 + 1).unwrap(), *(obj.1 .0));
                    relation_order.push(*obj.1 .1);
                }

                Ok((
                    UnfinishedRow {
                        missing: set.len(),
                        objects: Default::default(),
                        fields,
                        filters,
                        root_object: variant.root_object,
                        relation_order,
                    },
                    set,
                ))
            })
            .collect()
    }

    fn build_filters(
        &self,
        variant: &UnfinishedRowVariant,
        filters: &Filter,
    ) -> Result<FilterSource> {
        Ok(match filters {
            Filter::SimpleFilter(SimpleFilter {
                filter: SimpleFilterKind::Equals(EqualsFilter { lhs, rhs }),
            }) => FilterSource::Equals {
                lhs: self.build_filter_value(variant, lhs)?,
                rhs: self.build_filter_value(variant, rhs)?,
            },
            Filter::ComplexFilter(ComplexFilter { operator, operands }) => {
                let operands = operands
                    .iter()
                    .map(|op| self.build_filters(variant, op))
                    .collect::<Result<_>>()?;
                FilterSource::Complex {
                    operator: *operator,
                    operands,
                }
            }
        })
    }

    fn build_filter_value(
        &self,
        variant: &UnfinishedRowVariant,
        filter_value: &FilterValue,
    ) -> Result<FilterValueSource> {
        Ok(match filter_value {
            FilterValue::SchemaField(SchemaFieldFilter {
                schema_id,
                field_path,
            }) => {
                let object = match NonZeroU8::new(*schema_id) {
                    None => variant.root_object,
                    Some(relation_id) => *variant.objects.get(&relation_id).with_context(|| {
                        format!(
                            "Could not find a relation {} in view definition",
                            relation_id
                        )
                    })?,
                };
                FilterValueSource::SchemaField {
                    field_path: field_path.clone(),
                    object,
                }
            }
            FilterValue::ViewPath(ViewPathFilter { field_path }) => FilterValueSource::ViewPath {
                field_path: field_path.clone(),
            },
            FilterValue::RawValue(RawValueFilter { value }) => FilterValueSource::RawValue {
                value: value.0.clone(),
            },
            FilterValue::Computed(ComputedFilter { computation }) => {
                let computation = self.build_computation(variant, computation)?;
                FilterValueSource::Computed { computation }
            }
        })
    }

    fn build_fields(
        &self,
        variant: &UnfinishedRowVariant,
        fields: &HashMap<String, FieldDefinition>,
    ) -> Result<HashMap<String, FieldDefinitionSource>> {
        fields
            .iter()
            .map(|(field_name, field)| {
                self.build_field(variant, field)
                    .map(|field| (field_name.clone(), field))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    fn relations(view: &'a FullView) -> HashMap<Uuid, NonZeroU8> {
        view.relations
            .iter()
            .flat_map(|rel| flat_relation(rel))
            .map(|rel| (rel.global_id, rel.local_id))
            .collect()
    }

    fn build_field(
        &self,
        variant: &UnfinishedRowVariant,
        field: &FieldDefinition,
    ) -> Result<FieldDefinitionSource> {
        Ok(match field {
            FieldDefinition::Simple {
                field_name,
                field_type,
            } => FieldDefinitionSource::Simple {
                object: variant.root_object,
                field_name: field_name.clone(),
                field_type: field_type.clone(),
            },
            FieldDefinition::Computed {
                computation,
                field_type,
            } => {
                let computation = self.build_computation(variant, computation)?;
                FieldDefinitionSource::Computed {
                    computation,
                    field_type: field_type.clone(),
                }
            }
            FieldDefinition::SubObject { base, fields } => {
                let relation_id = NonZeroU8::new(*base).context(
                    "SubObject field type needs a reference to relation in view definition",
                )?;

                let object = *variant.objects.get(&relation_id).with_context(|| {
                    format!(
                        "Could not find a relation {} in view definition",
                        relation_id
                    )
                })?;

                let mut variant = variant.clone();
                variant.root_object = object;

                FieldDefinitionSource::SubObject {
                    fields: self.build_fields(&variant, fields)?,
                }
            }
        })
    }

    fn build_computation(
        &self,
        variant: &UnfinishedRowVariant,
        computation: &Computation,
    ) -> Result<ComputationSource> {
        Ok(match computation {
            Computation::RawValue(RawValueComputation { value }) => ComputationSource::RawValue {
                value: value.0.clone(),
            },
            Computation::FieldValue(FieldValueComputation {
                schema_id,
                field_path,
            }) => {
                let object = match NonZeroU8::new(*schema_id) {
                    None => variant.root_object,
                    Some(relation_id) => *variant.objects.get(&relation_id).with_context(|| {
                        format!(
                            "Could not find a relation {} in view definition",
                            relation_id
                        )
                    })?,
                };

                ComputationSource::FieldValue {
                    object,
                    field_path: field_path.clone(),
                }
            }
            Computation::Equals(EqualsComputation { lhs, rhs }) => {
                let lhs = self.build_computation(variant, lhs)?;
                let rhs = self.build_computation(variant, rhs)?;

                ComputationSource::Equals {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }
            }
        })
    }
}
