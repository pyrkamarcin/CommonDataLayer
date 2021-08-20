use std::{collections::HashMap, convert::TryFrom, ops::Deref};

use itertools::Itertools;
use rpc::common::RowDefinition as RPCRowDefinition;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct RowDefinition {
    pub(crate) object_ids: Vec<Uuid>,
    pub(crate) fields: HashMap<String, Value>,
}

impl TryFrom<RPCRowDefinition> for RowDefinition {
    type Error = anyhow::Error;

    fn try_from(row: RPCRowDefinition) -> Result<Self, Self::Error> {
        let object_ids = row
            .objects
            .into_iter()
            .map(|o| o.object_id.parse())
            .collect::<Result<_, _>>()?;
        let fields = row
            .fields
            .into_iter()
            .map(|(key, field)| {
                let field = serde_json::from_str(&field)?;
                Ok((key, field))
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(RowDefinition { object_ids, fields })
    }
}

impl RowDefinition {
    pub fn sha256_hash(&self) -> impl Deref<Target = [u8]> {
        let mut hash = Sha256::new();
        for id in self.object_ids.iter().sorted() {
            hash.update(id.as_bytes());
        }
        hash.finalize()
    }
}
