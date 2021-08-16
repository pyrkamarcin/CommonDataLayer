use std::{collections::HashMap, sync::Arc};

use settings_utils::apps::api::ApiSettings;
use tokio::sync::Mutex;

use crate::events::{EventStream, EventSubscriber};

#[derive(Clone)]
pub struct MQEvents {
    pub events: Arc<Mutex<HashMap<String, EventSubscriber>>>,
}

impl MQEvents {
    pub async fn subscribe_on_communication_method(
        &self,
        topic: &str,
        settings: &ApiSettings,
    ) -> anyhow::Result<EventStream> {
        tracing::debug!("subscribe on message queue: {}", topic);

        let mut event_map = self.events.lock().await;
        match event_map.get(topic) {
            Some(subscriber) => {
                let stream = subscriber.subscribe();
                Ok(stream)
            }
            None => {
                let kafka_events = self.events.clone();
                let (subscriber, stream) =
                    EventSubscriber::new(settings, topic, move |topic| async move {
                        tracing::warn!("Message queue stream has closed");
                        // Remove topic from hashmap so next time someone ask about this stream,
                        // it will be recreated
                        kafka_events.lock().await.remove(&topic);
                    })
                    .await?;
                event_map.insert(topic.into(), subscriber);
                Ok(stream)
            }
        }
    }
}
