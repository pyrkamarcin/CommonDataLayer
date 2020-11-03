use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

pub use crate::communication::error::Error;
pub use crate::communication::message::GenericMessage;
use crate::communication::resolution::Resolution;
use crate::report::ReportService;

mod error;
mod message;

pub mod resolution;

const DEFAULT_WAIT_TIME: Duration = Duration::from_secs(12);

pub struct ReceivedMessageBundle {
    pub msg: GenericMessage,
    pub status_sender: oneshot::Sender<Resolution>,
}

#[derive(Clone)]
pub struct MessageRouter {
    sender: Arc<Mutex<mpsc::Sender<ReceivedMessageBundle>>>,
    report_service: Arc<ReportService>,
    output_plugin: &'static str,
}

impl MessageRouter {
    pub fn new(
        sender: mpsc::Sender<ReceivedMessageBundle>,
        report_service: ReportService,
        output_plugin: &'static str,
    ) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender)),
            report_service: Arc::new(report_service),
            output_plugin,
        }
    }

    pub async fn handle_message(&self, msg: GenericMessage) -> Result<(), Error> {
        let (status_sender, status_receiver) = oneshot::channel::<Resolution>();

        self.sender
            .lock()
            .await
            .send_timeout(
                ReceivedMessageBundle { msg, status_sender },
                DEFAULT_WAIT_TIME,
            )
            .await
            .map_err(|_| Error::FailedToSend)?;

        let status = status_receiver.await.map_err(|_| Error::SenderDropped)?;

        match status {
            Resolution::StorageLayerFailure {
                ref description,
                ref object_id,
            } => {
                self.report_service
                    .report_failure(self.output_plugin, &description, *object_id)
                    .await
                    .map_err(Error::ReportingError)?;
            }
            Resolution::UserFailure {
                ref description,
                ref object_id,
                ref context,
            } => {
                self.report_service
                    .report_failure(
                        self.output_plugin,
                        &format!("{}; caused by `{}`", description, context),
                        *object_id,
                    )
                    .await
                    .map_err(Error::ReportingError)?;
            }
            Resolution::CommandServiceFailure { ref object_id } => {
                self.report_service
                    .report_failure(self.output_plugin, "Internal server error", *object_id)
                    .await
                    .map_err(Error::ReportingError)?;
            }
            Resolution::Success => {}
        }

        Ok(())
    }
}
