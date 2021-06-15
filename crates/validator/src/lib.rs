use communication_utils::parallel_consumer::ParallelConsumerHandler;
use communication_utils::message::CommunicationMessage;
use cdl_dto::ingestion::BorrowedInsertMessage;
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;

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

impl ParallelConsumerHandler for Handler {
    async fn handle<'a>(&'a self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let message: BorrowedInsertMessage = serde_json::from_str(msg.payload()?)?;
        let mut sr: SchemaRegistryClient<Channel> = rpc::schema_registry::connect(&self.schema_registry_url)?;

        let schema = sr.get_schema_definition(message.schema_id).await?;
    }
}
