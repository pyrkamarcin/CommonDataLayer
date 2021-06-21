use crate::settings::ValidatorContainer;
use anyhow::{Context, Error};
use async_trait::async_trait;
use cache::DynamicCache;
use cdl_dto::ingestion::{BorrowedInsertMessage, DataRouterInsertMessage};
use communication_utils::{
    get_order_group_id, message::CommunicationMessage, parallel_consumer::ParallelConsumerHandler,
    publisher::CommonPublisher,
};
use metrics_utils::{self as metrics, counter};
use misc_utils::current_timestamp;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tracing::{error, trace};
use utils::parallel_task_queue::ParallelTaskQueue;
use uuid::Uuid;

pub struct Handler {
    pub cache: Arc<Mutex<DynamicCache<Uuid, String>>>,
    pub producer: Arc<CommonPublisher>,
    pub task_queue: Arc<ParallelTaskQueue>,
    pub validator: Arc<Mutex<Box<dyn ValidatorContainer + Send + Sync>>>,
}

#[async_trait]
impl ParallelConsumerHandler for Handler {
    #[tracing::instrument(skip(self, message))]
    async fn handle<'a>(&'a self, message: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let order_group_id = get_order_group_id(message);
        let _guard =
            order_group_id.map(move |x| async move { self.task_queue.acquire_permit(x).await });

        trace!(
            "Received message ({:?}) `{:?}`",
            message.key(),
            message.payload()
        );

        let message_key = get_order_group_id(message).unwrap_or_default();
        counter!("cdl.data-router.input-msg", 1);
        let result = async {
            let payload = message.payload()?;

            let mut validator = self.validator.lock().await;

            if matches!(payload.chars().find(|c| !c.is_whitespace()), Some('[')) {
                trace!("Processing multimessage");

                let maybe_array: Vec<DataRouterInsertMessage> = serde_json::from_str(payload)
                    .context(
                        "Payload deserialization failed, message is not a valid cdl message ",
                    )?;

                let mut result = Ok(());

                for entry in maybe_array.iter() {
                    let r = self
                        .validate_and_send(&message_key, &mut validator, &entry)
                        .await;

                    if r.is_err() {
                        result = r;
                    }
                }

                result
            } else {
                trace!("Processing single message");

                let owned: DataRouterInsertMessage =
                    serde_json::from_str::<DataRouterInsertMessage>(payload).context(
                        "Payload deserialization failed, message is not a valid cdl message",
                    )?;

                self.validate_and_send(&message_key, &mut validator, &owned)
                    .await
            }
        }
        .await;

        counter!("cdl.data-router.input-request", 1);

        if let Err(error) = result {
            counter!("cdl.data-router.error", 1);

            return Err(error);
        } else {
            counter!("cdl.data-router.success", 1);
        }

        Ok(())
    }
}

#[tracing::instrument(skip(cache, producer))]
async fn route(
    cache: &Mutex<DynamicCache<Uuid, String>>,
    event: &DataRouterInsertMessage<'_>,
    message_key: &str,
    producer: &CommonPublisher,
) -> anyhow::Result<()> {
    let payload = BorrowedInsertMessage {
        object_id: event.object_id,
        schema_id: event.schema_id,
        timestamp: current_timestamp(),
        data: event.data,
    };

    let mut cache = cache.lock().await;
    let insert_destination = cache.get(event.schema_id).await?;

    send_message(
        producer,
        &insert_destination,
        message_key,
        serde_json::to_vec(&payload)?,
    )
    .await;
    Ok(())
}

#[tracing::instrument(skip(producer))]
async fn send_message(
    producer: &CommonPublisher,
    insert_destination: &str,
    key: &str,
    payload: Vec<u8>,
) {
    let payload_len = payload.len();
    let delivery_status = producer
        .publish_message(&insert_destination, key, payload)
        .await;

    if delivery_status.is_err() {
        error!(
            "Fatal error, delivery status for message not received.  Insert destination: `{}`, Key: `{}`, Payload len: `{}`, {:?}",
            insert_destination, key, payload_len, delivery_status
        );
    } else {
        counter!("cdl.data-router.output-singleok", 1);
    }
}

impl Handler {
    async fn validate_and_send(
        &self,
        message_key: &str,
        validator: &mut MutexGuard<'_, Box<dyn ValidatorContainer + Send + Sync>>,
        owned: &'_ DataRouterInsertMessage<'_>,
    ) -> Result<(), Error> {
        match validator.validate_value(owned.schema_id, owned.data).await {
            Ok(resolution) => {
                if resolution {
                    let r = route(&self.cache, &owned, &message_key, &self.producer)
                        .await
                        .context("Tried to send message and failed");
                    counter!("cdl.data-router.input-singlemsg", 1);
                    counter!("cdl.data-router.processed", 1);

                    r
                } else {
                    counter!("cdl.data-router.invalid", 1);
                    Ok(())
                }
            }
            Err(error) => Err(error),
        }
    }
}
