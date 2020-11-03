use std::fmt;

use async_trait::async_trait;
use structopt::StructOpt;
use tokio::sync::mpsc;

use error::OutputError;
pub use psql::{PostgresOutputConfig, PostgresOutputPlugin};
pub use sleigh::{SleighOutputConfig, SleighOutputPlugin};

use crate::communication::ReceivedMessageBundle;
use crate::output::druid::{DruidOutputConfig, DruidOutputPlugin};
use crate::output::victoria_metrics::config::VictoriaMetricsConfig;
use crate::output::victoria_metrics::VictoriaMetricsOutputPlugin;

mod druid;
mod error;
mod psql;
mod sleigh;
mod victoria_metrics;

#[derive(Clone, Debug, StructOpt)]
pub enum OutputArgs {
    Sleigh(SleighOutputConfig),
    Postgres(PostgresOutputConfig),
    Druid(DruidOutputConfig),
    VictoriaMetrics(VictoriaMetricsConfig),
}

pub struct Output {
    plugin: Box<dyn OutputPlugin + Send + Sync>,
    receiver: mpsc::Receiver<ReceivedMessageBundle>,
    sender: mpsc::Sender<ReceivedMessageBundle>,
}

#[async_trait]
pub trait OutputPlugin {
    async fn handle_message(
        &self,
        recv_msg_bundle: ReceivedMessageBundle,
    ) -> Result<(), OutputError>;
    fn name(&self) -> &'static str;
}

impl Output {
    pub async fn new(args: OutputArgs) -> Result<Self, OutputError> {
        let (tx, rx) = mpsc::channel::<ReceivedMessageBundle>(1024);

        let plugin: Box<dyn OutputPlugin + Send + Sync> = match args {
            OutputArgs::Sleigh(sleigh_config) => {
                Box::new(SleighOutputPlugin::new(sleigh_config).await?)
            }
            OutputArgs::Postgres(postgres_config) => {
                Box::new(PostgresOutputPlugin::new(postgres_config).await?)
            }
            OutputArgs::Druid(druid_config) => {
                Box::new(DruidOutputPlugin::new(druid_config).await?)
            }
            OutputArgs::VictoriaMetrics(victoria_config) => {
                Box::new(VictoriaMetricsOutputPlugin::new(victoria_config)?)
            }
        };

        Ok(Self {
            plugin,
            receiver: rx,
            sender: tx,
        })
    }

    pub fn channel(&self) -> mpsc::Sender<ReceivedMessageBundle> {
        self.sender.clone()
    }

    pub fn name(&self) -> &'static str {
        self.plugin.name()
    }

    pub async fn run(mut self) -> Result<(), OutputError> {
        while let Some(recv_msg_bundle) = self.receiver.recv().await {
            self.plugin.handle_message(recv_msg_bundle).await?
        }

        Ok(())
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.plugin.name())
    }
}
