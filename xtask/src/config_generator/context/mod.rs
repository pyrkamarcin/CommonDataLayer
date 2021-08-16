use std::path::PathBuf;

use dialoguer_trait::Dialogue;
use settings_utils::apps::PostgresSettings;

use crate::config_generator::defaults::{DEFAULT_KAFKA_BROKERS, DEFAULT_VICTORIA_METRICS_HOST};

pub mod api;
pub mod command_service;
pub mod data_router;
pub mod edge_registry;
pub mod materializer_general;
pub mod materializer_ondemand;
pub mod object_builder;
pub mod partial_update_engine;
pub mod query_router;
pub mod query_service;
pub mod query_service_ts;
pub mod schema_registry;

pub struct Context {
    pub(crate) communication: Communication,
    pub(crate) repo: Repo,
    pub(crate) postgres: PostgresContext,
    pub(crate) target_dir: PathBuf,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            communication: Default::default(),
            repo: Default::default(),
            postgres: Default::default(),
            target_dir: ".cdl".into(),
        }
    }
}

#[derive(Dialogue, Clone)]
pub struct PostgresContext {
    #[dialogue(prompt = "postgres host")]
    pub host: String,
    #[dialogue(prompt = "postgres port")]
    pub port: u16,
    #[dialogue(prompt = "postgres username")]
    pub username: String,
    #[dialogue(prompt = "postgres password")]
    pub password: String,
    #[dialogue(prompt = "postgres database name")]
    pub dbname: String,
    #[dialogue(prompt = "postgres schema")]
    pub schema: String,
}

#[derive(Dialogue)]
pub enum Communication {
    #[dialogue(prompt = "Kafka context")]
    Kafka(KafkaCommunication),
    #[dialogue(prompt = "RabbitMQ context")]
    Amqp(AmqpCommunication),
    #[dialogue(prompt = "Grpc")]
    Grpc,
}

#[derive(Dialogue)]
pub enum Repo {
    #[dialogue(prompt = "postgres repository")]
    Postgres,
    #[dialogue(prompt = "specify victoria metrics configuration")]
    VictoriaMetrics(VictoriaMetricsContext),
    #[dialogue(prompt = "specify druid configuration")]
    Druid(DruidContext),
}

#[derive(Dialogue)]
pub struct VictoriaMetricsContext {
    #[dialogue(prompt = "victoria metrics http://host:port url")]
    pub(crate) url: String,
}

#[derive(Dialogue)]
pub struct DruidContext {
    #[dialogue(prompt = "kafka topic druid is set up to listen for new messages")]
    pub(crate) topic: String,
    #[dialogue(prompt = "druid broker or router url")]
    pub(crate) url: String,
    #[dialogue(prompt = "table name where data is stored in druid")]
    pub(crate) table_name: String,
}

#[derive(Dialogue)]
pub struct AmqpCommunication {
    #[dialogue(prompt = "AMQP exchange eg. amqp://user:password@host:port/%2f")]
    pub(crate) exchange_url: String,
}

#[derive(Dialogue)]
pub struct KafkaCommunication {
    #[dialogue(prompt = "Kafka comma separated brokers")]
    pub(crate) brokers: String,
}

pub trait FromContext: Sized {
    fn from_context(context: &Context) -> anyhow::Result<Self>;
}

impl Default for VictoriaMetricsContext {
    fn default() -> Self {
        Self {
            url: DEFAULT_VICTORIA_METRICS_HOST.to_string(),
        }
    }
}

impl Default for Repo {
    fn default() -> Self {
        Self::Postgres
    }
}

impl Default for DruidContext {
    fn default() -> Self {
        Self {
            topic: "cdl.druid.data".to_string(),
            url: "http://localhost:8082/druid/v2".to_string(),
            table_name: "test".to_string(),
        }
    }
}

impl Default for Communication {
    fn default() -> Self {
        Self::Kafka(KafkaCommunication {
            brokers: DEFAULT_KAFKA_BROKERS.to_string(),
        })
    }
}

impl From<PostgresContext> for PostgresSettings {
    fn from(context: PostgresContext) -> Self {
        PostgresSettings {
            host: context.host,
            port: context.port,
            username: context.username,
            password: context.password,
            dbname: context.dbname,
            schema: context.schema,
        }
    }
}

impl Default for PostgresContext {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "1234".to_string(),
            dbname: "postgres".to_string(),
            schema: "public".to_string(),
        }
    }
}
