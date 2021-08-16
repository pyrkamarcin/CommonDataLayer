use std::{collections::HashMap, sync::Arc};

use cache::DynamicCache;
use notification_utils::{IntoSerialize, NotificationPublisher};
use plugins::{MaterializerPlugin, PostgresMaterializer};
use rpc::{
    common::RowDefinition,
    materializer_general::{
        general_materializer_server::GeneralMaterializer,
        Empty,
        MaterializedView,
        Options,
    },
};
use serde::Serialize;
use settings_utils::apps::PostgresSettings;
use view::ViewSupplier;

use crate::view::ViewCache;

mod plugins;
mod view;

#[derive(Serialize, Clone)]
pub struct MaterializationNotification {
    view_id: String,
    options: String,
    rows: Vec<MaterializationRow>,
}

#[derive(Serialize, Clone)]
struct MaterializationRow {
    objects: Vec<Object>,
    fields: HashMap<String, String>,
}

#[derive(Serialize, Clone)]
struct Object {
    object_id: String,
    schema_id: String,
}

impl From<RowDefinition> for MaterializationRow {
    fn from(row: RowDefinition) -> Self {
        Self {
            objects: row
                .objects
                .into_iter()
                .map(|o| Object {
                    object_id: o.object_id,
                    schema_id: o.schema_id,
                })
                .collect(),
            fields: row.fields,
        }
    }
}

impl IntoSerialize<MaterializationNotification> for MaterializedView {
    fn into_serialize(self) -> MaterializationNotification {
        MaterializationNotification {
            view_id: self.view_id,
            options: self.options.options,
            rows: self
                .rows
                .into_iter()
                .map(MaterializationRow::from)
                .collect(),
        }
    }
}

type MaterializerNotificationPublisher =
    Arc<NotificationPublisher<MaterializedView, MaterializationNotification>>;

pub struct MaterializerImpl {
    materializer: Arc<dyn MaterializerPlugin>,
    notification_publisher: MaterializerNotificationPublisher,
    view_cache: ViewCache,
}

impl MaterializerImpl {
    pub async fn new(
        args: PostgresSettings,
        notification_publisher: MaterializerNotificationPublisher,
        schema_registry_url: String,
        cache_size: usize,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            materializer: Arc::new(PostgresMaterializer::new(&args).await?),
            notification_publisher,
            view_cache: DynamicCache::new(cache_size, ViewSupplier::new(schema_registry_url)),
        })
    }
}

#[tonic::async_trait]
impl GeneralMaterializer for MaterializerImpl {
    #[tracing::instrument(skip(self))]
    async fn validate_options(
        &self,
        request: tonic::Request<Options>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let options: Options = request.into_inner();
        tracing::debug!(?options, "Options to validate");

        match self.validate_options_inner(&options.options) {
            Ok(_) => Ok(tonic::Response::new(Empty {})),
            Err(err) => Err(tonic::Status::invalid_argument(format!("{}", err))),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn upsert_view(
        &self,
        request: tonic::Request<MaterializedView>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let materialized_view = request.into_inner();

        let instance =
            Arc::clone(&self.notification_publisher).and_message_body(&materialized_view);

        let upsert_res = self
            .upsert_view_inner(materialized_view)
            .await
            .map_err(|err| {
                tracing::error!("Materialization error` {:?}", err);
                tonic::Status::internal(format!("{}", err))
            });

        let notification_res = match upsert_res.as_ref() {
            Ok(()) => instance.notify("success").await,
            Err(e) => instance.notify(&format!("{:?}", e)).await,
        };

        if let Err(err) = notification_res {
            tracing::error!("Failed to send notification {:?}", err)
        }

        upsert_res.map(|_| tonic::Response::new(Empty {}))
    }
}

impl MaterializerImpl {
    fn validate_options_inner(&self, options: &str) -> anyhow::Result<()> {
        let options: serde_json::Value = serde_json::from_str(options)?;
        self.materializer.validate_options(options)?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn upsert_view_inner(
        &self,
        materialized_view: MaterializedView,
    ) -> Result<(), anyhow::Error> {
        tracing::debug!(?materialized_view, "materialized view");

        let view_id = materialized_view.view_id.parse()?;

        let view_definition = self.view_cache.get(view_id).await?;

        self.materializer
            .upsert_view(materialized_view, view_definition.clone())
            .await?;

        Ok(())
    }
}
