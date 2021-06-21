#![feature(trait_alias, async_closure)]

use anyhow::Context;
use cache::{CacheSupplier, DynamicCache};
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
use std::sync::Arc;
use uuid::Uuid;

pub struct Handler {
    schema_registry_url: Arc<String>,
}

impl Handler {
    pub fn new(schema_registry_url: Arc<String>) -> Self {
        Self {
            schema_registry_url,
        }
    }
}

#[async_trait::async_trait]
impl ParallelConsumerHandler for Handler {
    async fn handle<'a>(&'a self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        // let message: BorrowedInsertMessage = serde_json::from_str(msg.payload()?)?;
        // let mut sr: SchemaRegistryClient<Channel> =
        //     rpc::schema_registry::connect(self.schema_registry_url.to_owned()).await?;
        //
        // let def = sr
        //     .get_schema_definition(VersionedId {
        //         id: message.schema_id.to_string(),
        //         version_req: None,
        //     })
        //     .await?
        //     .into_inner();
        //
        // let schema =
        //     jsonschema::JSONSchema::compile(&serde_json::from_slice(def.definition.as_slice())?)?;
        //
        // if let Err(err) = schema.validate(&serde_json::from_str(message.data.get())?) {
        //     println!("Oh no ");
        // }

        Ok(())
    }
}

// https://github.com/Stranger6667/jsonschema-rs/issues/145
// JSON schema isn't owned, so we have to store it's value along the way - self referential mess
pub struct Validator {
    cache: DynamicCache<Uuid, JSONSchema<'static>>,
}

pub struct ValidatorCacheSupplier {
    schema_retriever: Box<
        dyn Fn(Uuid) -> Pin<Box<dyn Future<Output = anyhow::Result<&'static Value>> + Send + Sync>>
            + Send
            + Sync,
    >,
}

#[async_trait::async_trait]
impl CacheSupplier<Uuid, JSONSchema<'static>> for ValidatorCacheSupplier {
    async fn retrieve(&self, key: Uuid) -> anyhow::Result<JSONSchema<'static>> {
        let schema = (self.schema_retriever)(key).await?;
        let json_schema = JSONSchema::compile(schema).context("Couldn't convert schema")?;
        Ok(json_schema)
    }
}

impl ValidatorCacheSupplier {
    fn new(
        schema_retriever: Box<
            dyn Fn(
                    Uuid,
                )
                    -> Pin<Box<dyn Future<Output = anyhow::Result<&'static Value>> + Send + Sync>>
                + Send
                + Sync,
        >,
    ) -> Self {
        Self { schema_retriever }
    }
}

impl Validator {
    pub fn new(
        cache_capacity: usize,
        get_schema: Box<
            dyn Fn(
                    Uuid,
                )
                    -> Pin<Box<dyn Future<Output = anyhow::Result<&'static Value>> + Send + Sync>>
                + Send
                + Sync,
        >,
    ) -> Self {
        Self {
            cache: DynamicCache::new(
                cache_capacity,
                Box::new(ValidatorCacheSupplier::new(get_schema)),
            ),
        }
    }
}
