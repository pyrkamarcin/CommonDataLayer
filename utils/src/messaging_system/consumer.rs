use anyhow::Context;
use async_stream::try_stream;
use futures_util::stream::{Stream, StreamExt};
use lapin::{options::BasicConsumeOptions, types::FieldTable, Channel, Connection};
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    ClientConfig,
};
use std::sync::Arc;
use tokio_amqp::LapinTokioExt;

use super::{
    message::CommunicationMessage, message::KafkaCommunicationMessage,
    message::RabbitCommunicationMessage, CommunicationResult,
};

pub enum CommonConsumer {
    Kafka {
        consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
    },
    RabbitMq {
        _connection: Box<Connection>,
        _channel: Box<Channel>,
        consumer: lapin::Consumer,
    },
}
impl CommonConsumer {
    pub async fn new_kafka(
        group_id: &str,
        brokers: &str,
        topics: &[&str],
    ) -> CommunicationResult<CommonConsumer> {
        let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
            .set("group.id", &group_id)
            .set("bootstrap.servers", &brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Consumer creation failed")?;

        rdkafka::consumer::Consumer::subscribe(&consumer, topics)
            .context("Can't subscribe to specified topics")?;

        Ok(CommonConsumer::Kafka {
            consumer: Arc::new(consumer),
        })
    }

    pub async fn new_rabbit(
        connection_string: &str,
        consumer_tag: &str,
        queue_name: &str,
    ) -> CommunicationResult<CommonConsumer> {
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = connection.create_channel().await?;
        let consumer = channel
            .basic_consume(
                queue_name,
                consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        Ok(CommonConsumer::RabbitMq {
            _channel: Box::new(channel),
            _connection: Box::new(connection),
            consumer,
        })
    }

    pub async fn consume(
        &mut self,
    ) -> impl Stream<Item = CommunicationResult<Box<dyn CommunicationMessage + '_>>> {
        try_stream! {
        match self {
            CommonConsumer::Kafka { consumer } => {
                let mut message_stream = consumer.start();
                    while let Some(message) = message_stream.next().await {
                        let message = message?;
                        yield Box::new(KafkaCommunicationMessage{message,consumer:consumer.clone()}) as Box<dyn CommunicationMessage>;
                    }
                }
                CommonConsumer::RabbitMq {
                    consumer,
                    _connection,
                    _channel,
                } => {
                    while let Some(message) = consumer.next().await {
                        let message = message?;
                        yield Box::new(RabbitCommunicationMessage{channel:message.0, delivery:message.1})as Box<dyn CommunicationMessage>;
                    }
                }
            }
        }
    }
}
