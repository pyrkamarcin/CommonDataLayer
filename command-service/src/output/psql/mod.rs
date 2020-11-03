use std::time;

use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::tokio_postgres::types::Json;
use bb8_postgres::tokio_postgres::NoTls;
use bb8_postgres::PostgresConnectionManager;
use log::{error, trace};
use serde_json::Value;

pub use config::PostgresOutputConfig;
pub use error::Error;

use crate::communication::resolution::Resolution;
use crate::communication::{GenericMessage, ReceivedMessageBundle};
use crate::output::error::OutputError;
use crate::output::OutputPlugin;
use utils::metrics::counter;
use utils::status_endpoints;

pub mod config;
pub mod error;

pub struct PostgresOutputPlugin {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PostgresOutputPlugin {
    pub async fn new(config: PostgresOutputConfig) -> Result<Self, Error> {
        let manager = bb8_postgres::PostgresConnectionManager::new(
            config.url.parse().map_err(Error::FailedToParseUrl)?,
            NoTls,
        );
        let pool = bb8::Pool::builder()
            .max_size(20)
            .connection_timeout(time::Duration::from_secs(120))
            .build(manager)
            .await
            .map_err(Error::FailedToConnect)?;

        Ok(Self { pool })
    }

    async fn store_message(
        pool: Pool<PostgresConnectionManager<NoTls>>,
        msg: GenericMessage,
    ) -> Resolution {
        let connection = pool.get().await.unwrap();
        let payload: Value = match serde_json::from_slice(&msg.payload) {
            Ok(json) => json,
            Err(_err) => {
                return Resolution::CommandServiceFailure {
                    object_id: msg.object_id,
                }
            }
        };

        let store_result = connection
            .query(
                "INSERT INTO data (object_id, version, schema_id, payload)
                 VALUES ($1, $2, $3, $4)",
                &[
                    &msg.object_id,
                    &msg.timestamp,
                    &msg.schema_id,
                    &Json(payload),
                ],
            )
            .await;

        trace!("PSQL `INSERT` {:?}", store_result);

        match store_result {
            Ok(_) => {
                counter!("cdl.command-service.store.psql", 1);

                Resolution::Success
            }
            Err(err) => Resolution::StorageLayerFailure {
                description: err.to_string(),
                object_id: msg.object_id,
            },
        }
    }
}

#[async_trait]
impl OutputPlugin for PostgresOutputPlugin {
    async fn handle_message(
        &self,
        recv_msg_bundle: ReceivedMessageBundle,
    ) -> Result<(), OutputError> {
        // Assumption is that db is provisioned
        let pool = self.pool.clone();

        tokio::spawn(async move {
            let msg = recv_msg_bundle.msg;

            trace!("Storing message {:?}", msg);
            let resolution = PostgresOutputPlugin::store_message(pool, msg).await;

            if recv_msg_bundle.status_sender.send(resolution).is_err() {
                error!("Failed to send status to report service");
                status_endpoints::mark_as_unhealthy();
            }
        });

        Ok(())
    }

    fn name(&self) -> &'static str {
        "PostgreSQL"
    }
}
