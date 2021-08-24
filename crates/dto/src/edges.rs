use std::convert::TryInto;

use rpc::{common::types::LogicOperator, edge_registry::types::SimpleFilterSide};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{RequestError, RequestResult, ResponseResult, TryFromRpc, TryIntoRpc};

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct RelationTree {
    pub rows: Vec<RelationTreeRow>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RelationTreeRow {
    pub base_object_id: Uuid,
    pub relation_object_ids: Vec<Uuid>,
}

impl RelationTree {
    pub fn from_rpc(rpc: rpc::edge_registry::RelationTreeResponse) -> RequestResult<Self> {
        Ok(Self {
            rows: rpc
                .rows
                .into_iter()
                .map(RelationTreeRow::from_rpc)
                .collect::<RequestResult<Vec<_>>>()?,
        })
    }
}

impl RelationTreeRow {
    pub fn from_rpc(rpc: rpc::edge_registry::RelationTreeRow) -> RequestResult<Self> {
        Ok(Self {
            base_object_id: rpc.base_object_id.parse()?,
            relation_object_ids: rpc
                .relation_object_ids
                .into_iter()
                .map(|f| f.parse())
                .collect::<Result<_, _>>()?,
        })
    }
}

/// View's filter
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[serde(rename_all = "snake_case")]
#[rpc(
    rpc = "rpc::edge_registry::Filter",
    tag = "filter_kind",
    tag_rpc = "rpc::edge_registry::filter::FilterKind"
)]
pub enum Filter {
    #[rpc(rename = "Simple")]
    SimpleFilter(SimpleFilter),
    #[rpc(rename = "Complex")]
    ComplexFilter(ComplexFilter),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleFilter {
    pub side: SimpleFilterSide,
    pub relation: u8,
    pub ids: Vec<Uuid>,
}

impl TryFromRpc<rpc::edge_registry::SimpleFilter> for SimpleFilter {
    fn try_from_rpc(rpc: rpc::edge_registry::SimpleFilter) -> RequestResult<Self> {
        Ok(Self {
            side: rpc.side.try_into()?,
            relation: rpc.relation as u8,
            ids: rpc
                .ids
                .into_iter()
                .map(|f| f.parse())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryIntoRpc for SimpleFilter {
    type Rpc = rpc::edge_registry::SimpleFilter;

    fn try_into_rpc(self) -> ResponseResult<Self::Rpc> {
        Ok(rpc::edge_registry::SimpleFilter {
            side: self.side.into(),
            relation: self.relation as u32,
            ids: self.ids.into_iter().map(|f| f.to_string()).collect(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ComplexFilter {
    pub operator: LogicOperator,
    pub operands: Vec<Filter>,
}

impl TryFromRpc<rpc::edge_registry::ComplexFilter> for ComplexFilter {
    fn try_from_rpc(rpc: rpc::edge_registry::ComplexFilter) -> RequestResult<Self> {
        Ok(Self {
            operator: rpc.operator.try_into()?,
            operands: rpc
                .operands
                .into_iter()
                .map(TryFromRpc::try_from_rpc)
                .collect::<RequestResult<_>>()?,
        })
    }
}

impl TryIntoRpc for ComplexFilter {
    type Rpc = rpc::edge_registry::ComplexFilter;

    fn try_into_rpc(self) -> ResponseResult<Self::Rpc> {
        Ok(rpc::edge_registry::ComplexFilter {
            operator: self.operator.into(),
            operands: self
                .operands
                .into_iter()
                .map(|o| o.try_into_rpc())
                .collect::<ResponseResult<_>>()?,
        })
    }
}
