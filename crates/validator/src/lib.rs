#![feature(trait_alias, async_closure)]

use cache::{DynamicCache, ProduceCacheItem};
use cdl_dto::ingestion::BorrowedInsertMessage;
use communication_utils::message::CommunicationMessage;
use communication_utils::parallel_consumer::ParallelConsumerHandler;
use jsonschema::JSONSchema;
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::schema_registry::VersionedId;
use rpc::tonic::transport::Channel;
use serde_json::Value;
use std::error::Error;
use std::future::Future;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;
use uuid::Uuid;

pub trait GetSchema<'a> = Fn(&Uuid) -> Pin<Box<dyn Future<Output = anyhow::Result<&'a Value>>>>;

pub struct Handler {
    schema_registry_url: String,
}

impl Handler {
    pub fn new(schema_registry_url: String) -> Self {
        Self {
            schema_registry_url,
        }
    }
}

#[async_trait::async_trait]
impl ParallelConsumerHandler for Handler {
    async fn handle<'a>(&'a self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let message: BorrowedInsertMessage = serde_json::from_str(msg.payload()?)?;
        let mut sr: SchemaRegistryClient<Channel> =
            rpc::schema_registry::connect(self.schema_registry_url.to_owned()).await?;

        let def = sr
            .get_schema_definition(VersionedId {
                id: message.schema_id.to_string(),
                version_req: None,
            })
            .await?
            .into_inner();

        let schema =
            jsonschema::JSONSchema::compile(&serde_json::from_slice(def.definition.as_slice())?)?;

        if let Err(err) = schema.validate(&serde_json::from_str(message.data.get())?) {
            println!("Oh no ");
        }

        Ok(())
    }
}

struct Schema<'a> {
    validator: JSONSchema<'a>,
}

// https://github.com/Stranger6667/jsonschema-rs/issues/145
// JSON schema isn't owned, so we have to store it's value along the way - self referential mess
pub struct Validator<'a> {
    cache: DynamicCache<Uuid, JSONSchema<'a>>,
}

impl<'a> Validator<'a> {
    pub fn new(cache_capacity: usize, get_schema: impl GetSchema<'a> + 'static) -> Self {
        let on_missing = move |id| {
            let fut = async {
                let schema = JSONSchema::compile(get_schema(id).await?)?;
                Ok(schema)
            };
            Box::pin(fut) as Pin<Box<dyn Future<Output = anyhow::Result<JSONSchema>>>>
        };
        Self {
            cache: DynamicCache::new(cache_capacity, on_missing)
        }
    }
}
