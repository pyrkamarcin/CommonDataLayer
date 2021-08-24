use anyhow::Result;

use crate::{sources::RowSource, RowDefinition};

mod field_builder;

use field_builder::FieldBuilder;
use row_filter::RowFilter;

mod row_filter;

pub struct RowBuilder {}

impl RowBuilder {
    pub fn new() -> Self {
        Self {}
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn build(&self, source: RowSource) -> Result<Option<RowDefinition>> {
        let field_builder = FieldBuilder {
            objects: &source.objects,
        };

        let fields = source
            .fields
            .iter()
            .map(|field| field_builder.build(field))
            .collect::<anyhow::Result<_>>()?;

        RowFilter::new(&source.objects).filter(
            RowDefinition {
                objects: source.relation_order,
                fields,
            },
            source.filters,
        )
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::Result;
    use misc_utils::serde_json::{to_string_sorted, SortSettings};
    use serde_json::Value;

    use super::*;
    use crate::{buffer_stream::ObjectBuffer, object_id_pair::ObjectIdPair, view_plan::ViewPlan};

    #[test]
    fn test_row_builder() -> Result<()> {
        snapshot_runner::test_snapshots("built_rows", |input| {
            let view = input.get_json("view").expect("view");
            let tree_response = input.get_json("tree_response").expect("tree_response");
            let relation_map = input.get_json("relation_map").expect("relation_map");
            let objects: BTreeMap<ObjectIdPair, Value> =
                input.get_json("objects").expect("could not get objects");

            let view_plan =
                ViewPlan::try_new(view, tree_response, relation_map).expect("valid view plan");
            let mut buffer = ObjectBuffer::new(view_plan);
            let row_builder = RowBuilder::new();

            let rows: Vec<RowDefinition> = objects
                .into_iter()
                .filter_map(|(id, value)| buffer.add_object(id, value))
                .collect::<Result<Vec<_>>>()
                .expect("row sources")
                .into_iter()
                .flatten()
                .flat_map(|row| row_builder.build(row).transpose())
                .collect::<Result<Vec<_>>>()
                .expect("built rows");

            to_string_sorted(
                &rows,
                SortSettings {
                    pretty: true,
                    sort_arrays: true,
                },
            )
            .expect("Cannot serialize")
        })
    }
}
