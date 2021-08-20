use std::convert::{TryFrom, TryInto};

use cdl_dto::materialization::{ElasticsearchMaterializerOptions, FullView};
use elasticsearch::{http::transport::Transport, Elasticsearch, IndexParts};
use rpc::materializer_general::MaterializedView;
use serde_json::Value;

use crate::plugins::{models::RowDefinition, MaterializerPlugin};

pub struct ElasticsearchMaterializer {
    client: Elasticsearch,
}

#[derive(Debug)]
struct ElasticsearchView {
    options: ElasticsearchMaterializerOptions,
    rows: Vec<RowDefinition>,
}

impl TryFrom<MaterializedView> for ElasticsearchView {
    type Error = anyhow::Error;

    fn try_from(view: MaterializedView) -> Result<Self, Self::Error> {
        let options = serde_json::from_str(&view.options.options)?;

        let rows = view
            .rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<anyhow::Result<_>>()?;

        Ok(ElasticsearchView { options, rows })
    }
}

#[async_trait::async_trait]
impl MaterializerPlugin for ElasticsearchMaterializer {
    fn validate_options(&self, _: Value) -> anyhow::Result<()> {
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn upsert_view(&self, view: MaterializedView, _: FullView) -> anyhow::Result<()> {
        let view: ElasticsearchView = view.try_into()?;
        for row in view.rows {
            self.client
                .index(IndexParts::IndexId(
                    &view.options.index_name,
                    &hex::encode(&*row.sha256_hash()),
                ))
                .body(&row.fields)
                .send()
                .await?;
        }

        Ok(())
    }
}

impl ElasticsearchMaterializer {
    pub fn new(node_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            client: Elasticsearch::new(Transport::single_node(node_url)?),
        })
    }
}
