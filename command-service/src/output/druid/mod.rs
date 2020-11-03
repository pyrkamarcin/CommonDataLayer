use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use log::error;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;

use utils::metrics::counter;

use crate::communication::resolution::Resolution;
use crate::communication::{GenericMessage, ReceivedMessageBundle};
pub use crate::output::druid::config::DruidOutputConfig;
pub use crate::output::druid::error::Error;
use crate::output::error::OutputError;
use crate::output::OutputPlugin;
use utils::status_endpoints;

mod config;
mod error;

pub struct DruidOutputPlugin {
    producer: FutureProducer,
    topic: Arc<String>,
}

#[async_trait]
impl OutputPlugin for DruidOutputPlugin {
    async fn handle_message(
        &self,
        recv_msg_bundle: ReceivedMessageBundle,
    ) -> Result<(), OutputError> {
        let producer = self.producer.clone();
        let topic = Arc::clone(&self.topic);

        tokio::spawn(async move {
            let msg = recv_msg_bundle.msg;
            let resolution = DruidOutputPlugin::store_message(producer, msg, topic.as_str()).await;

            if recv_msg_bundle.status_sender.send(resolution).is_err() {
                error!("Failed to send status to report service");
                status_endpoints::mark_as_unhealthy();
            }
        });

        Ok(())
    }

    fn name(&self) -> &'static str {
        "Druid timeseries"
    }
}

impl DruidOutputPlugin {
    pub async fn new(args: DruidOutputConfig) -> Result<Self, Error> {
        Ok(Self {
            producer: ClientConfig::new()
                .set("bootstrap.servers", &args.brokers)
                .set("message.timeout.ms", "5000")
                .create()
                .map_err(Error::ProducerCreation)?,
            topic: Arc::new(args.topic),
        })
    }

    async fn store_message(
        producer: FutureProducer,
        msg: GenericMessage,
        topic: &str,
    ) -> Resolution {
        let key = msg.object_id.to_string();
        let record = FutureRecord {
            topic: &topic,
            partition: None,
            payload: Some(&msg.payload),
            key: Some(&key),
            timestamp: Some(msg.timestamp),
            headers: None,
        };

        match producer.send(record, Duration::from_secs(0)).await {
            Err((err, _)) => Resolution::StorageLayerFailure {
                description: err.to_string(),
                object_id: msg.object_id,
            },
            Ok(_) => {
                counter!("cdl.command-service.store.druid", 1);

                Resolution::Success
            }
        }
    }
}
