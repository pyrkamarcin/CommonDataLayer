use rpc::common::RowDefinition as RpcRowDefinition;
use rpc::materializer_general::{MaterializedView as RpcMaterializedView, Options};
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use uuid::Uuid;

pub use object_id_pair::ObjectIdPair;

mod object_id_pair;

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MaterializedView {
    pub view_id: Uuid,
    pub options: Value,
    pub rows: Vec<RowDefinition>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct RowDefinition {
    pub object_ids: HashSet<Uuid>,
    pub fields: HashMap<String, Value>,
}

impl TryInto<RpcRowDefinition> for RowDefinition {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<RpcRowDefinition, Self::Error> {
        let fields = self
            .fields
            .into_iter()
            .map(|(key, value)| Ok((key, serde_json::to_string(&value)?)))
            .collect::<serde_json::Result<_>>()?;
        Ok(RpcRowDefinition {
            object_ids: self
                .object_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            fields,
        })
    }
}

impl TryInto<RpcMaterializedView> for MaterializedView {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<RpcMaterializedView, Self::Error> {
        let rows = self
            .rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<serde_json::Result<_>>()?;

        Ok(RpcMaterializedView {
            view_id: self.view_id.to_string(),
            options: Options {
                options: serde_json::to_string(&self.options)?,
            },
            rows,
        })
    }
}
