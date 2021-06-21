use cache::{CacheSupplier, DynamicCache};
use cdl_dto::ingestion::BorrowedInsertMessage;
use communication_utils::message::CommunicationMessage;
use communication_utils::parallel_consumer::ParallelConsumerHandler;
use communication_utils::publisher::CommonPublisher;
use jsonschema::JSONSchema;
use ouroboros::self_referencing;
use rpc::schema_registry::VersionedId;
use rpc::tonic::Request;
use serde_json::{to_value, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct Handler {
    validator: Arc<Mutex<Validator>>,
    producer: Arc<CommonPublisher>,
    send_to: String,
}

impl Handler {
    pub fn new(validator: Validator, producer: Arc<CommonPublisher>, send_to: String) -> Self {
        Self {
            validator: Arc::new(Mutex::new(validator)),
            producer,
            send_to,
        }
    }
}

#[async_trait::async_trait]
impl ParallelConsumerHandler for Handler {
    async fn handle<'a>(&'a self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let payload = msg.payload()?;
        let message: BorrowedInsertMessage = serde_json::from_str(payload)?;

        let mut validator = self.validator.lock().await;

        if validator
            .validate_value(message.schema_id, &to_value(message.data)?)
            .await?
        {
            self.producer
                .publish_message(&self.send_to, msg.key()?, payload.as_bytes().to_vec())
                .await?;
        } else {
            tracing::trace!(
                "value '{}' is not valid cdl object of schema `{}`",
                message.data,
                message.schema_id,
            )
        }

        Ok(())
    }
}

pub struct Validator {
    cache: DynamicCache<Uuid, Schema>,
}

#[self_referencing]
pub struct Schema {
    pub json: Box<Value>,
    #[borrows(json)]
    #[covariant]
    pub validator: JSONSchema<'this>,
}

pub struct ValidatorCacheSupplier {
    schema_registry_url: String,
}

#[async_trait::async_trait]
impl CacheSupplier<Uuid, Schema> for ValidatorCacheSupplier {
    async fn retrieve(&self, key: Uuid) -> anyhow::Result<Schema> {
        let json = get_schema_from_registry(key, self.schema_registry_url.clone()).await?;
        Ok(SchemaTryBuilder {
            json: Box::new(json),
            validator_builder: |json: &Value| JSONSchema::compile(json),
        }
        .try_build()?)
    }
}

impl ValidatorCacheSupplier {
    fn new(schema_registry_url: String) -> Self {
        Self {
            schema_registry_url,
        }
    }
}

impl Validator {
    pub fn new(cache_capacity: usize, schema_registry_url: String) -> Self {
        Self {
            cache: DynamicCache::new(
                cache_capacity,
                Box::new(ValidatorCacheSupplier::new(schema_registry_url)),
            ),
        }
    }

    pub async fn validate_value(&mut self, id: Uuid, value: &Value) -> anyhow::Result<bool> {
        let schema = self.cache.get(id).await?;
        Ok(schema.is_valid(value))
    }
}

impl Schema {
    pub fn schema(&self) -> &Value {
        self.borrow_json().as_ref()
    }

    pub fn is_valid(&self, other: &Value) -> bool {
        self.borrow_validator().is_valid(other)
    }
}
pub async fn get_schema_from_registry(
    id: Uuid,
    schema_registry_url: String,
) -> anyhow::Result<Value> {
    let mut conn = rpc::schema_registry::connect(schema_registry_url).await?;
    let schema_def = conn
        .get_schema_definition(Request::new(VersionedId {
            id: id.to_string(),
            version_req: None,
        }))
        .await?
        .into_inner();
    Ok(serde_json::from_slice(&schema_def.definition)?)
}
