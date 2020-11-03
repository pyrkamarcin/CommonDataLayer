use super::{KafkaConfig, ReplicationEvent};
use log::{error, info};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::{
    process,
    sync::{mpsc, Arc},
    time::Duration,
};
use tokio::{runtime::Handle, sync::oneshot};

pub fn replicate_db_events(
    config: KafkaConfig,
    recv: mpsc::Receiver<ReplicationEvent>,
    tokio_runtime: Handle,
    mut kill_signal: oneshot::Receiver<()>,
) {
    let producer = Arc::new(build_kafka_producer(&config));
    loop {
        let event = recv.recv().unwrap_or_else(|_e| {
            error!("Fatal error, synchronization channel closed.");
            process::abort();
        });
        if kill_signal.try_recv().is_ok() {
            info!("Master replication disabled");
            return;
        };

        tokio_runtime
            .enter(|| send_messages_to_kafka(producer.clone(), config.topics[0].clone(), event));
    }
}

fn send_messages_to_kafka(
    producer: Arc<FutureProducer>,
    topic_name: String,
    event: ReplicationEvent,
) {
    let key = match &event {
        ReplicationEvent::AddSchema { id, .. } => id,
        ReplicationEvent::AddSchemaVersion { id, .. } => id,
        ReplicationEvent::AddViewToSchema { schema_id, .. } => schema_id,
        ReplicationEvent::UpdateSchemaName { id, .. } => id,
        ReplicationEvent::UpdateSchemaTopic { id, .. } => id,
        ReplicationEvent::UpdateSchemaQueryAddress { id, .. } => id,
        ReplicationEvent::UpdateView { id, .. } => id,
    };
    let serialized = serde_json::to_string(&event).unwrap();
    let serialized_key = key.to_string();
    tokio::spawn(async move {
        let delivery_status = producer.send(
            FutureRecord::to(&topic_name)
                .payload(&serialized)
                .key(&serialized_key),
            Duration::from_secs(1),
        );
        if delivery_status.await.is_err() {
            error!("Fatal error, delivery status for message not received.");
            process::abort();
        }
    });
}

pub fn build_kafka_producer(config: &KafkaConfig) -> FutureProducer {
    // https://kafka.apache.org/documentation/#producerconfigs
    // TODO: should connect to kafka and check if connection was successful before reporting service as started
    //       (otherwise there is no way of knowing that kafka broker is unreachable)
    ClientConfig::new()
        .set("bootstrap.servers", &config.brokers)
        .set("message.timeout.ms", "5000")
        .set("acks", "all")
        .set("compression.type", "none")
        .set("max.in.flight.requests.per.connection", "1")
        .create()
        .expect("Producer creation error")
}
