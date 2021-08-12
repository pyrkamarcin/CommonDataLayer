use settings_utils::apps::{
    command_service::{
        CommandServiceAmqpSettings,
        CommandServiceDruidSettings,
        CommandServiceGRpcSettings,
        CommandServiceKafkaSettings,
        CommandServiceListenerSettings,
        CommandServiceRepositoryKind,
        CommandServiceSettings,
        CommandServiceVictoriaMetricsSettings,
    },
    AmqpConsumeOptions,
    CommunicationMethod,
    LogSettings,
    MonitoringSettings,
    NotificationSettings,
};

use crate::config_generator::{
    context::{Communication, Context, FromContext, Repo},
    defaults::{DEFAULT_CDL_NOTIFICATION_CHANNEL, DEFAULT_COMMAND_SERVICE_LISTEN_URL},
};

pub const COMMAND_SERVICE_NAMESPACE: &str = "command_service";

impl FromContext for CommandServiceSettings {
    fn from_context(context: &Context) -> anyhow::Result<Self> {
        let mut settings = Self {
            communication_method: CommunicationMethod::Kafka,
            repository_kind: CommandServiceRepositoryKind::Postgres,
            async_task_limit: 32,
            postgres: None,
            victoria_metrics: None,
            druid: None,
            kafka: None,
            amqp: None,
            grpc: None,
            notifications: NotificationSettings {
                enabled: true,
                destination: DEFAULT_CDL_NOTIFICATION_CHANNEL.to_string(),
            },
            listener: CommandServiceListenerSettings {
                ordered_sources: "cdl.document.ordered.1.data".to_string(),
                unordered_sources: "cdl.document.unordered.1.data".to_string(),
            },
            monitoring: MonitoringSettings {
                metrics_port: 0,
                status_port: 0,
                otel_service_name: COMMAND_SERVICE_NAMESPACE.to_string(),
            },
            log: LogSettings {
                rust_log: format!("info,{}=debug", COMMAND_SERVICE_NAMESPACE),
            },
        };

        match context.communication {
            Communication::Kafka(ref kafka) => {
                settings.communication_method = CommunicationMethod::Kafka;
                settings.kafka = Some(CommandServiceKafkaSettings {
                    brokers: kafka.brokers.clone(),
                    group_id: COMMAND_SERVICE_NAMESPACE.to_string(),
                });
            }
            Communication::Amqp(ref amqp) => {
                settings.communication_method = CommunicationMethod::Amqp;
                settings.amqp = Some(CommandServiceAmqpSettings {
                    exchange_url: amqp.exchange_url.to_string(),
                    tag: COMMAND_SERVICE_NAMESPACE.to_string(),
                    consume_options: Some(AmqpConsumeOptions {
                        no_local: false,
                        no_ack: false,
                        exclusive: false,
                        nowait: false,
                    }),
                })
            }
            Communication::Grpc => {
                settings.communication_method = CommunicationMethod::Grpc;
                settings.grpc = Some(CommandServiceGRpcSettings {
                    address: DEFAULT_COMMAND_SERVICE_LISTEN_URL.parse()?,
                })
            }
        }

        match context.repo {
            Repo::Postgres => {
                settings.repository_kind = CommandServiceRepositoryKind::Postgres;
                settings.postgres = Some(context.postgres.clone().into());
            }
            Repo::VictoriaMetrics(ref vm) => {
                settings.repository_kind = CommandServiceRepositoryKind::VictoriaMetrics;
                settings.victoria_metrics = Some(CommandServiceVictoriaMetricsSettings {
                    url: vm.url.parse()?,
                })
            }
            Repo::Druid(ref druid) => {
                settings.repository_kind = CommandServiceRepositoryKind::Druid;
                settings.druid = Some(CommandServiceDruidSettings {
                    topic: druid.topic.to_string(),
                })
            }
        }

        Ok(settings)
    }
}
