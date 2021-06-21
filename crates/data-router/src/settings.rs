use serde::{Deserialize, Serialize};

use communication_utils::{parallel_consumer::ParallelCommonConsumer, publisher::CommonPublisher};
use serde_json::value::RawValue;
use settings_utils::{
    AmqpSettings, ConsumerKafkaSettings, GRpcSettings, LogSettings, MonitoringSettings,
};
use task_utils::task_limiter::TaskLimiter;
use uuid::Uuid;
use validator::Validator;

#[derive(Deserialize, Debug, Serialize)]
pub struct Settings {
    pub communication_method: CommunicationMethod,
    pub routing_cache_capacity: usize,
    #[serde(default = "default_async_task_limit")]
    pub async_task_limit: usize,

    pub kafka: Option<ConsumerKafkaSettings>,
    pub amqp: Option<AmqpSettings>,
    pub grpc: Option<GRpcSettings>,

    pub monitoring: MonitoringSettings,

    pub services: ServicesSettings,

    pub log: LogSettings,

    pub validation: Option<ValidationSettings>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationMethod {
    Kafka,
    Amqp,
    #[serde(rename = "grpc")]
    GRpc,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ServicesSettings {
    pub schema_registry_url: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ValidationSettings {
    pub enabled: bool,
    pub validation_cache_capacity: usize,
}

const fn default_async_task_limit() -> usize {
    32
}

impl Settings {
    pub async fn consumer(&self) -> anyhow::Result<ParallelCommonConsumer> {
        match (
            &self.kafka,
            &self.amqp,
            &self.grpc,
            &self.communication_method,
        ) {
            (Some(kafka), _, _, CommunicationMethod::Kafka) => {
                kafka
                    .parallel_consumer(TaskLimiter::new(self.async_task_limit))
                    .await
            }
            (_, Some(amqp), _, CommunicationMethod::Amqp) => {
                amqp.parallel_consumer(TaskLimiter::new(self.async_task_limit))
                    .await
            }
            (_, _, Some(grpc), CommunicationMethod::GRpc) => grpc.parallel_consumer().await,
            _ => anyhow::bail!("Unsupported consumer specification"),
        }
    }

    pub async fn producer(&self) -> anyhow::Result<CommonPublisher> {
        Ok(
            match (
                &self.kafka,
                &self.amqp,
                &self.grpc,
                &self.communication_method,
            ) {
                (Some(kafka), _, _, CommunicationMethod::Kafka) => {
                    CommonPublisher::new_kafka(&kafka.brokers).await?
                }
                (_, Some(amqp), _, CommunicationMethod::Amqp) => {
                    CommonPublisher::new_amqp(&amqp.exchange_url).await?
                }
                (_, _, Some(_), CommunicationMethod::GRpc) => CommonPublisher::new_grpc().await?,
                _ => anyhow::bail!("Unsupported consumer specification"),
            },
        )
    }

    pub fn validator(&self) -> Box<dyn ValidatorContainer + Send + Sync> {
        if let Some(validation) = &self.validation {
            if validation.enabled {
                return Box::new(ObjectValidator {
                    inner: Validator::new(
                        validation.validation_cache_capacity,
                        self.services.schema_registry_url.clone(),
                    ),
                });
            }
        }
        Box::new(EmptyValidator)
    }
}

#[async_trait::async_trait]
pub trait ValidatorContainer {
    async fn validate_value(&mut self, schema_id: Uuid, value: &RawValue) -> anyhow::Result<bool>;
}

pub struct EmptyValidator;

pub struct ObjectValidator {
    inner: Validator,
}

#[async_trait::async_trait]
impl ValidatorContainer for EmptyValidator {
    async fn validate_value(&mut self, _: Uuid, _: &RawValue) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[async_trait::async_trait]
impl ValidatorContainer for ObjectValidator {
    async fn validate_value(&mut self, schema_id: Uuid, value: &RawValue) -> anyhow::Result<bool> {
        self.inner
            .validate_value(schema_id, &serde_json::from_str(value.get())?)
            .await
    }
}
