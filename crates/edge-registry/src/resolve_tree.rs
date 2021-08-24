use anyhow::Context;
use bb8_postgres::{
    bb8::PooledConnection,
    tokio_postgres::{types::ToSql, NoTls},
    PostgresConnectionManager,
};
use cdl_dto::{edges::Filter, materialization::Relation, TryFromRpc};
use itertools::Itertools;
use rpc::{
    common::types::{LogicOperator, SearchFor},
    edge_registry::{types::SimpleFilterSide, RelationTreeResponse, RelationTreeRow, TreeQuery},
};
use uuid::Uuid;

#[tracing::instrument(skip(connection))]
pub async fn resolve_tree_impl(
    connection: PooledConnection<'_, PostgresConnectionManager<NoTls>>,
    mut request: TreeQuery,
) -> anyhow::Result<RelationTreeResponse> {
    let mut params: Vec<Uuid> = vec![];

    let base_relation =
        Relation::try_from_rpc(request.relations.pop().context("No relations provided")?)?;

    let (r0_column_path, r1_column_path) = match base_relation.search_for {
        SearchFor::Parents => ("r1.child_object_id", "r1.parent_object_id"),
        SearchFor::Children => ("r1.parent_object_id", "r1.child_object_id"),
    };
    let mut select = format!(
        "SELECT {} as r0_id, {} as r1_id",
        r0_column_path, r1_column_path
    );

    params.push(base_relation.global_id);
    let mut where_ = format!("WHERE r1.relation_id = ${}", params.len());

    let mut joins = "".to_owned();

    for relation in base_relation.relations.into_iter() {
        add_join(
            relation,
            r1_column_path,
            &mut select,
            &mut joins,
            &mut params,
        );
    }

    for relation in request
        .relations
        .into_iter()
        .map(|relation| Relation::try_from_rpc(relation))
    {
        let relation = relation?;
        add_join(
            relation,
            r0_column_path,
            &mut select,
            &mut joins,
            &mut params,
        );
    }

    if let Some(filter) = request.filters {
        let filter = Filter::try_from_rpc(filter)?;
        where_.push_str(&format!(" AND {}", add_filter(filter)));
    }

    let query = format!("{} FROM edges as r1{} {}", select, joins, where_);
    tracing::trace!(?query, ?params, "Generated query");

    trait ToSqlSync = ToSql + std::marker::Sync;
    let params = params
        .iter()
        .map(|uuid| uuid as &dyn ToSqlSync)
        .collect::<Vec<_>>();

    let rows = connection
        .query(query.as_str(), params.as_slice())
        .await?
        .into_iter()
        .map(|row| RelationTreeRow {
            base_object_id: row.get::<_, Uuid>(0).to_string(),
            relation_object_ids: {
                let columns = row.columns().len();
                let mut ids = Vec::with_capacity(columns - 1);
                for i in 1..columns {
                    ids.push(row.get::<_, Uuid>(i).to_string());
                }
                ids
            },
        })
        .collect();

    Ok(RelationTreeResponse { rows })
}

fn add_filter(filter: Filter) -> String {
    match filter {
        Filter::SimpleFilter(filter) => {
            let ids = filter.ids.into_iter().map(|f| format!("'{}'", f)).join(",");
            match filter.side {
                SimpleFilterSide::InParentObjIds => {
                    format!(" r{}.parent_object_id in ({})", filter.relation, ids)
                }
                SimpleFilterSide::InChildObjIds => {
                    format!(" r{}.child_object_id in ({})", filter.relation, ids)
                }
            }
        }
        Filter::ComplexFilter(filter) => {
            let operands = match filter.operator {
                LogicOperator::And => filter.operands.into_iter().map(add_filter).join(" AND "),
                LogicOperator::Or => filter.operands.into_iter().map(add_filter).join(" OR "),
            };
            format!("({})", operands)
        }
    }
}

fn add_join(
    relation: Relation,
    parent_column_path: &str,
    select: &mut String,
    joins: &mut String,
    params: &mut Vec<Uuid>,
) {
    let (select_column_path, join_column_path) = match relation.search_for {
        SearchFor::Children => (
            format!("r{}.child_object_id", relation.local_id),
            format!("r{}.parent_object_id", relation.local_id),
        ),
        SearchFor::Parents => (
            format!("r{}.parent_object_id", relation.local_id),
            format!("r{}.child_object_id", relation.local_id),
        ),
    };

    select.push_str(format!(", {} as r{}_id", select_column_path, relation.local_id).as_str());

    let join_condition = format!("{}  = {}", join_column_path, parent_column_path);

    params.push(relation.global_id);
    joins.push_str(
        format!(
            " JOIN edges as r{} on r{}.relation_id = ${} AND {}",
            relation.local_id,
            relation.local_id,
            params.len(),
            join_condition
        )
        .as_str(),
    );

    for sub_relation in relation.relations {
        add_join(sub_relation, &select_column_path, select, joins, params);
    }
}
