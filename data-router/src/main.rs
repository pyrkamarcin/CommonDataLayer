use std::sync::{Arc, Mutex};
use std::{process, time::Duration};

use anyhow::Context;
use futures_util::stream::StreamExt;
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties, Consumer,
};
use log::error;
use lru_cache::LruCache;
use rdkafka::{
    message::OwnedHeaders,
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use schema_registry::connect_to_registry;
use schema_registry::rpc::schema::Id;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_amqp::*;
use utils::{
    abort_on_poison,
    metrics::{self, counter},
};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize)]
struct Config {
    pub input_addr: String,
    pub input_queue: String,
    pub kafka_brokers: String,
    pub kafka_error_channel: String,
    pub schema_registry_addr: String,
    pub cache_capacity: usize,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputData {
    pub schema_id: Uuid,
    pub object_id: Uuid,
    pub data: Value,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = Arc::new(envy::from_env::<Config>().context("Env vars not set correctly")?);

    let (_conn, _channel, mut consumer) =
        create_rabbitmq_consumer(&config.input_addr, &config.input_queue).await?;
    let producer = Arc::new(build_kafka_producer(&config.kafka_brokers));
    let cache = Arc::new(Mutex::new(LruCache::new(config.cache_capacity)));

    let message_handler = tokio::spawn(async move {
        while let Some(delivery) = consumer.next().await {
            let cache = cache.clone();
            let producer = producer.clone();
            let config = config.clone();

            tokio::spawn(async move {
                let (channel, delivery) = delivery.unwrap();

                let result = handle_input(
                    &delivery,
                    &cache,
                    &config.schema_registry_addr,
                    producer.clone(),
                )
                .await;

                counter!("cdl.data-router.input-request", 1);

                if let Err(error) = result {
                    error!("{:?}", error);
                    send_kafka_error(
                        producer,
                        config.kafka_error_channel.clone(),
                        format!("{:?}", error),
                    );
                }
                let ack_result = channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await;
                if let Err(e) = ack_result {
                    error!(
                        "Fatal error, delivery status for message not received. {:?}",
                        e
                    );
                    process::abort();
                }
            });
        }
    });

    metrics::serve();
    message_handler.await?;

    Ok(())
}
async fn create_rabbitmq_consumer(
    addr: &str,
    queue_name: &str,
) -> anyhow::Result<(Connection, Channel, Consumer)> {
    let conn = Connection::connect(addr, ConnectionProperties::default().with_tokio()).await?;
    let channel = conn.create_channel().await?;

    // channel
    //     .queue_declare(
    //         queue_name,
    //         QueueDeclareOptions::default(),
    //         FieldTable::default(),
    //     )
    //     .await?;

    let consumer = channel
        .basic_consume(
            queue_name,
            "CDL_DATA_ROUTER",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    Ok((conn, channel, consumer))
}

async fn handle_input(
    delivery: &Delivery,
    cache: &Mutex<LruCache<Uuid, String>>,
    schema_addr: &str,
    producer: Arc<FutureProducer>,
) -> anyhow::Result<()> {
    let json = std::str::from_utf8(&delivery.data).context("Payload was not valid UTF-8")?;
    let event: InputData = serde_json::from_str(json).context("Payload deserialization failed")?;

    let topic_name = get_schema_topic(cache, &event, schema_addr).await?;

    send_messages_to_kafka(producer, topic_name, event);
    Ok(())
}

async fn get_schema_topic(
    cache: &Mutex<LruCache<Uuid, String>>,
    event: &InputData,
    schema_addr: &str,
) -> anyhow::Result<String> {
    let recv_channel = cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .get_mut(&event.schema_id)
        .cloned();
    if let Some(val) = recv_channel {
        return Ok(val);
    }

    let mut client = connect_to_registry(schema_addr.to_owned()).await?;
    let channel = client
        .get_schema_topic(Id {
            id: event.schema_id.to_string(),
        })
        .await?
        .into_inner()
        .topic;
    cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .insert(event.schema_id, channel.clone());

    Ok(channel)
}

fn send_messages_to_kafka(producer: Arc<FutureProducer>, topic_name: String, value: InputData) {
    tokio::spawn(async move {
        let key = &value.object_id.to_string();
        let payload = &serde_json::to_vec(&value.data).unwrap_or_default();
        let delivery_status = producer.send(
            FutureRecord::to(&topic_name)
                .payload(payload)
                .headers(OwnedHeaders::new().add("SCHEMA_ID", &value.schema_id.to_string()))
                .key(key),
            Duration::from_secs(1),
        );

        if delivery_status.await.is_err() {
            error!("Fatal error, delivery status for message not received.");
            process::abort();
        };
    });
}

fn build_kafka_producer(brokers: &str) -> FutureProducer {
    // https://kafka.apache.org/documentation/#producerconfigs
    // TODO: should connect to kafka and check if connection was successful before reporting service as started
    //       (otherwise there is no way of knowing that kafka broker is unreachable)
    ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("message.timeout.ms", "5000")
        .set("acks", "all")
        .set("compression.type", "none")
        .set("max.in.flight.requests.per.connection", "1")
        .create()
        .expect("Producer creation error")
}

fn send_kafka_error(producer: Arc<FutureProducer>, topic_name: String, value: String) {
    tokio::spawn(async move {
        let delivery_status = producer.send(
            FutureRecord::to(&topic_name).payload(&value).key("error"),
            Duration::from_secs(1),
        );
        if delivery_status.await.is_err() {
            error!("Fatal error, delivery status for message not received.");
            process::abort();
        };
    });
}
