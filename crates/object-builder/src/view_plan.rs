use std::{collections::HashMap, num::NonZeroU8};

use anyhow::Result;
use cdl_dto::{
    edges::{RelationTree, RelationTreeRow},
    materialization::{self, FullView},
};
use itertools::Itertools;
use rpc::common::types::SearchFor;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use self::builder::ViewPlanBuilder;
use crate::{
    sources::{FieldDefinitionSource, FilterSource, RowSource},
    utils::flat_relation,
    ObjectIdPair,
};

mod builder;

#[derive(Clone, Debug)]
pub struct UnfinishedRowVariant {
    root_object: ObjectIdPair,
    objects: HashMap<NonZeroU8, ObjectIdPair>,
}
impl UnfinishedRowVariant {
    fn form_row(
        row: RelationTreeRow,
        view: &FullView,
        relation_map: &HashMap<Uuid, (Uuid, Uuid)>,
    ) -> Self {
        let root_object = ObjectIdPair {
            object_id: row.base_object_id,
            schema_id: view.base_schema_id,
        };
        let mut objects = HashMap::new();
        for object in view
            .relations
            .iter()
            .flat_map(flat_relation)
            .zip_eq(row.relation_object_ids)
        {
            let relation_map = relation_map
                .get(&object.0.global_id)
                .expect("Relation not pre-fetched");
            let schema_id = match object.0.search_for {
                SearchFor::Parents => relation_map.0,
                SearchFor::Children => relation_map.1,
            };
            objects.insert(
                object.0.local_id,
                ObjectIdPair {
                    object_id: object.1,
                    schema_id,
                },
            );
        }
        Self {
            root_object,
            objects,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct UnfinishedRow {
    /// Number of objects that are still missing to finish the join
    pub missing: usize,
    /// Stored objects waiting for missing ones
    pub objects: HashMap<ObjectIdPair, Value>,

    pub root_object: ObjectIdPair,

    pub fields: HashMap<String, FieldDefinitionSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<FilterSource>,

    /// Mapping schema_id, object_id pairs to proper relation identifier
    pub relation_order: Vec<ObjectIdPair>,
}

impl From<UnfinishedRow> for RowSource {
    fn from(unfinished: UnfinishedRow) -> Self {
        RowSource {
            objects: unfinished.objects,
            root_object: unfinished.root_object,
            fields: unfinished.fields,
            filters: unfinished.filters,
            relation_order: unfinished.relation_order,
        }
    }
}

/// Because objects are received on the go, and object builder needs to create joins,
/// these objects need to be tempoirairly stored in some kind of buffer until the last part
/// of the join arrives
#[derive(Debug, Serialize)]
pub struct ViewPlan {
    pub(crate) unfinished_rows: Vec<Option<UnfinishedRow>>,
    pub(crate) missing: HashMap<ObjectIdPair, Vec<usize>>, // (_, indices to unfinished_rows)
    #[serde(skip)] // Serialize is used only for tests, we dont need to assert view
    pub(crate) view: FullView,
}

impl ViewPlan {
    pub fn try_new(
        view: FullView,
        response: RelationTree,
        relation_map: HashMap<Uuid, (Uuid, Uuid)>,
    ) -> Result<Self> {
        let mut missing: HashMap<ObjectIdPair, Vec<usize>> = Default::default();

        let builder = ViewPlanBuilder::new(&view, relation_map);

        let unfinished_rows = builder
            .build_rows(response)?
            .into_iter()
            .enumerate()
            .map(|(idx, (row, set))| {
                for object in set {
                    missing.entry(object).or_default().push(idx);
                }
                Some(row)
            })
            .collect();

        Ok(ViewPlan {
            unfinished_rows,
            missing,
            view,
        })
    }

    pub fn objects_filter(&self) -> HashMap<Uuid, materialization::Schema> {
        self.missing
            .keys()
            .sorted_by_key(|o| o.schema_id)
            .group_by(|ObjectIdPair { schema_id, .. }| schema_id)
            .into_iter()
            .map(|(schema_id, objects)| {
                (
                    *schema_id,
                    materialization::Schema {
                        object_ids: objects.into_iter().map(|pair| pair.object_id).collect(),
                    },
                )
            })
            .collect()
    }

    pub fn builder(&self) -> ViewPlanBuilder {
        ViewPlanBuilder {
            view: &self.view,
            relations: Default::default(),
            relation_map: Default::default(),
        }
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use anyhow::Result;
    use misc_utils::serde_json::{to_string_sorted, SortSettings};

    use super::*;

    #[test]
    fn build_view_plan_test() -> Result<()> {
        snapshot_runner::test_snapshots("view_plan_build", |input| {
            let view = input.get_json("view").expect("view");
            let tree_response = input.get_json("tree_response").expect("tree_response");
            let relation_map = input.get_json("relation_map").expect("relation_map");

            let view_plan =
                ViewPlan::try_new(view, tree_response, relation_map).expect("valid view plan");

            to_string_sorted(
                &view_plan,
                SortSettings {
                    pretty: true,
                    sort_arrays: true,
                },
            )
            .expect("Cannot serialize")
        })
    }
}
