use super::{CustomContext, KafkaConfig, LoggingConsumer, ReplicationEvent};
use crate::db::SchemaDb;
use anyhow::Context;
use futures_util::stream::StreamExt;
use log::{error, info, trace};
use rdkafka::{
    consumer::{CommitMode, Consumer, StreamConsumer},
    message::BorrowedMessage,
    ClientConfig, Message,
};
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;

pub async fn consume_kafka_topic(
    consumer: StreamConsumer<CustomContext>,
    db: Arc<SchemaDb>,
    mut kill_signal: Receiver<()>,
) -> anyhow::Result<()> {
    let mut message_stream = consumer.start();

    while let Some(message) = message_stream.next().await {
        if kill_signal.try_recv().is_ok() {
            info!("Slave replication disabled");
            return Ok(());
        };
        trace!("Received message");
        match message {
            Err(error) => error!("Received malformed message: {}", error),
            Ok(message) => {
                if let Err(error) = consume_message(&consumer, message, &db).await {
                    error!("Error while processing message: {}", error);
                }
            }
        }
    }

    Ok(())
}

async fn consume_message(
    consumer: &LoggingConsumer,
    message: BorrowedMessage<'_>,
    db: &SchemaDb,
) -> anyhow::Result<()> {
    let payload = message
        .payload()
        .ok_or_else(|| anyhow::anyhow!("Message had no payload"))?;
    let json = std::str::from_utf8(payload).context("Payload is not valid UTF-8")?;
    let event: ReplicationEvent =
        serde_json::from_str(json).context("Payload deserialization failed")?;
    trace!("Consuming message: {:?}", event);
    match event {
        ReplicationEvent::AddSchema { id, schema } => {
            db.add_schema(schema, Some(id))?;
        }
        ReplicationEvent::AddSchemaVersion { id, new_version } => {
            db.add_new_version_of_schema(id, new_version)?;
        }
        ReplicationEvent::AddViewToSchema {
            schema_id,
            view,
            view_id,
        } => {
            db.add_view_to_schema(schema_id, view, Some(view_id))?;
        }
        ReplicationEvent::UpdateSchemaName { id, new_name } => {
            db.update_schema_name(id, new_name)?;
        }
        ReplicationEvent::UpdateSchemaTopic { id, new_topic } => {
            db.update_schema_topic(id, new_topic)?;
        }
        ReplicationEvent::UpdateSchemaQueryAddress {
            id,
            new_query_address,
        } => {
            db.update_schema_query_address(id, new_query_address)?;
        }
        ReplicationEvent::UpdateView { id, view } => {
            db.update_view(id, view)?;
        }
    };

    consumer
        .commit_message(&message, CommitMode::Async)
        .context("Failed to commit message")
}

pub fn build_kafka_consumer(config: &KafkaConfig) -> anyhow::Result<LoggingConsumer> {
    // https://kafka.apache.org/documentation/#consumerconfigs
    // https://docs.confluent.io/3.2.1/clients/librdkafka/CONFIGURATION_8md.html
    // TODO: should connect to kafka and check if connection was successful before reporting service as started
    //       (otherwise there is no way of knowing that kafka broker is unreachable)
    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", &config.group_id)
        .set("bootstrap.servers", &config.brokers)
        .set("enable.partition.eof", "false")
        // for synchronizing new storage instance should be true, after initial synchronization change to false - and 'enable' reads
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        //       If we're not storing full transaction log value should be none/error and initial stored should be given with initial db state(snapshot from another instance)
        .create_with_context(CustomContext)
        .context("Consumer creation failed")?;

    let topics = config
        .topics
        .iter()
        .map(|topic| topic.as_ref())
        .collect::<Vec<_>>();

    consumer
        .subscribe(topics.as_slice())
        .context("Can't subscribe to specified topics")?;

    Ok(consumer)
}
