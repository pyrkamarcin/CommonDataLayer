use std::fs;
use std::fs::Permissions;

use dialoguer_trait::Dialogue;

use anyhow::bail;
use context::{Communication, Context, FromContext, Repo};
use settings_utils::apps::api::ApiSettings;
use settings_utils::apps::command_service::CommandServiceSettings;
use settings_utils::apps::data_router::DataRouterSettings;
use settings_utils::apps::edge_registry::EdgeRegistrySettings;
use settings_utils::apps::materializer_general::MaterializerGeneralSettings;
use settings_utils::apps::materializer_ondemand::MaterializerOndemandSettings;
use settings_utils::apps::object_builder::ObjectBuilderSettings;
use settings_utils::apps::partial_update_engine::PartialUpdateEngineSettings;
use settings_utils::apps::query_router::QueryRouterSettings;
use settings_utils::apps::query_service::QueryServiceSettings;
use settings_utils::apps::query_service_ts::QueryServiceTsSettings;
use settings_utils::apps::schema_registry::SchemaRegistrySettings;

use crate::config_generator::context::{
    AmqpCommunication, DruidContext, KafkaCommunication, PostgresContext, VictoriaMetricsContext,
};
use pico_args::Arguments;
use serde::Serialize;
use std::ffi::OsString;
use std::path::Path;

mod context;
mod defaults;

#[derive(Dialogue)]
enum Options {
    #[dialogue(name = "review")]
    Review,
    #[dialogue(name = "update")]
    Update,
    #[dialogue(name = "save & quit")]
    SaveAndQuit,
    #[dialogue(name = "abort")]
    Abort,
}

#[derive(Dialogue)]
enum ContextUpdateAction {
    #[dialogue(name = "communication method")]
    CommunicationMethod,
    #[dialogue(name = "repository kind")]
    RepositoryKind,
}

#[derive(Dialogue)]
enum Application {
    #[dialogue(name = "api")]
    Api,
    #[dialogue(name = "command service")]
    CommandService,
    #[dialogue(name = "query service")]
    QueryService,
    #[dialogue(name = "query service ts")]
    QueryServiceTs,
    #[dialogue(name = "data router")]
    DataRouter,
    #[dialogue(name = "query router")]
    QueryRouter,
    #[dialogue(name = "materializer on demand")]
    MaterializerOndemand,
    #[dialogue(name = "materializer general")]
    MaterializerGeneral,
    #[dialogue(name = "partial update engine")]
    PartialUpdateEngine,
    #[dialogue(name = "object builder")]
    ObjectBuilder,
    #[dialogue(name = "edge registry")]
    EdgeRegistry,
    #[dialogue(name = "schema registry")]
    SchemaRegistry,
}

pub(crate) fn interactive() -> anyhow::Result<()> {
    let mut context = Context::default();

    loop {
        match Options::compose("Choose action")? {
            Options::Review => review(&context)?,
            Options::Update => update(&mut context)?,
            Options::SaveAndQuit => {
                let target_dir = dialoguer::Input::new()
                    .default(".cdl".to_string())
                    .show_default(true)
                    .with_prompt("target directory")
                    .interact_text()?;

                context.target_dir = target_dir.into();

                save(&context)?;
                break;
            }
            Options::Abort => break,
        }
    }

    Ok(())
}

pub(crate) fn from_args(args: Vec<OsString>) -> anyhow::Result<()> {
    let mut context = Context::default();

    let mut args = Arguments::from_vec(args);

    if let Some(method) = args.opt_value_from_str::<_, String>("--communication-method")? {
        context.communication = match method.as_str() {
            "kafka" => Communication::Kafka(KafkaCommunication {
                brokers: args.value_from_str("--brokers")?,
            }),
            "amqp" => Communication::Amqp(AmqpCommunication {
                exchange_url: args.value_from_str("--exchange-url")?,
            }),
            "grpc" => Communication::Grpc,
            _ => bail!("invalid communication-method"),
        };
    }

    if let Some(repository) = args.opt_value_from_str::<_, String>("--repository")? {
        match repository.as_str() {
            "postgres" => {
                context.repo = Repo::Postgres;
                context.postgres = PostgresContext {
                    host: args.value_from_str("--pg-host")?,
                    port: args.value_from_str("--pg-port")?,
                    username: args.value_from_str("--pg-username")?,
                    password: args.value_from_str("--pg-password")?,
                    dbname: args.value_from_str("--pg-dbname")?,
                    schema: args.value_from_str("--pg-schema")?,
                };
            }
            "victoria-metrics" => {
                context.repo = Repo::VictoriaMetrics(VictoriaMetricsContext {
                    url: args.value_from_str("--vm-url")?,
                })
            }
            "druid" => {
                context.repo = Repo::Druid(DruidContext {
                    topic: args.value_from_str("--druid-topic")?,
                    url: args.value_from_str("--druid-url")?,
                    table_name: args.value_from_str("--druid-table-name")?,
                })
            }
            _ => bail!("invalid repository"),
        }
    }

    if let Some(host) = args.opt_value_from_str("--pg-host")? {
        context.postgres.host = host;
    }

    if let Some(port) = args.opt_value_from_str("--pg-port")? {
        context.postgres.port = port;
    }

    if let Some(dbname) = args.opt_value_from_str("--pg-dbname")? {
        context.postgres.dbname = dbname;
    }

    if let Some(schema) = args.opt_value_from_str("--pg-schema")? {
        context.postgres.schema = schema;
    }

    if let Some(username) = args.opt_value_from_str("--pg-username")? {
        context.postgres.username = username;
    }

    if let Some(password) = args.opt_value_from_str("--pg-password")? {
        context.postgres.password = password;
    }

    if let Some(target_dir) = args.opt_value_from_str("--target-dir")? {
        context.target_dir = target_dir;
    }

    save(&context)?;

    Ok(())
}

fn save(context: &Context) -> anyhow::Result<()> {
    fs::create_dir_all(&context.target_dir)?;

    let mut apps = vec![
        "api",
        "data-router",
        "query-router",
        "schema-registry",
        "command-service",
    ];

    write_app_config::<ApiSettings, _>(context, context.target_dir.join("api.toml"))?;
    write_app_config::<DataRouterSettings, _>(
        context,
        context.target_dir.join("data-router.toml"),
    )?;
    write_app_config::<QueryRouterSettings, _>(
        context,
        context.target_dir.join("query-router.toml"),
    )?;
    write_app_config::<SchemaRegistrySettings, _>(
        context,
        context.target_dir.join("schema-registry.toml"),
    )?;
    write_app_config::<CommandServiceSettings, _>(
        context,
        context.target_dir.join("command-service.toml"),
    )?;

    match context.repo {
        Repo::Postgres => {
            println!("Generating query-service.toml");
            write_app_config::<QueryServiceSettings, _>(
                context,
                context.target_dir.join("query-service.toml"),
            )?;
            apps.push("query-service");
        }
        Repo::VictoriaMetrics(_) | Repo::Druid(_) => {
            println!("Generating query-service-ts.toml");
            write_app_config::<QueryServiceTsSettings, _>(
                context,
                context.target_dir.join("query-service-ts.toml"),
            )?;
            apps.push("query-service-ts");
        }
    }
    match context.communication {
        Communication::Kafka(_) => {
            println!("Generating materialization config");
            write_app_config::<EdgeRegistrySettings, _>(
                context,
                context.target_dir.join("edge-registry.toml"),
            )?;
            write_app_config::<ObjectBuilderSettings, _>(
                context,
                context.target_dir.join("object-builder.toml"),
            )?;
            write_app_config::<PartialUpdateEngineSettings, _>(
                context,
                context.target_dir.join("partial-update-engine.toml"),
            )?;
            write_app_config::<MaterializerGeneralSettings, _>(
                context,
                context.target_dir.join("materializer-general.toml"),
            )?;
            write_app_config::<MaterializerOndemandSettings, _>(
                context,
                context.target_dir.join("materializer-ondemand.toml"),
            )?;

            apps.push("edge-registry");
            apps.push("object-builder");
            apps.push("partial-update-engine");
            apps.push("materializer-general");
            apps.push("materializer-ondemand");
        }
        Communication::Amqp(_) | Communication::Grpc => {
            println!("Skipping materialization, unsupported communication backend")
        }
    }

    create_service_scripts(apps, &context.target_dir)?;

    println!("You can now run `{target_dir}/start.sh` and `{target_dir}/stop.sh` to manage your local environment", target_dir = context.target_dir.display());

    Ok(())
}

fn write_app_config<T: FromContext + Serialize, F: AsRef<Path>>(
    context: &Context,
    file: F,
) -> anyhow::Result<()> {
    Ok(fs::write(
        file,
        toml::to_string(&T::from_context(context)?)?,
    )?)
}

const START_FILE_BASE: &str = include_str!("start.sh.base");
const STOP_FILE_BASE: &str = include_str!("stop.sh.base");

fn create_service_scripts(apps: Vec<&'static str>, target_dir: &Path) -> anyhow::Result<()> {
    let mut start_script = START_FILE_BASE.to_string();
    let mut stop_script = STOP_FILE_BASE.to_string();

    for app in &apps {
        start_script.push_str(&format!(
            "./target/debug/{app} > \"$SCRIPT_DIR/.tmp/{app}.log\" 2>&1 &\n",
            app = app
        ));
        stop_script.push_str(&format!("killall {app} || echo \"{app}\"\n", app = app));
    }

    fs::write(target_dir.join("start.sh"), start_script)?;
    fs::write(target_dir.join("stop.sh"), stop_script)?;

    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(target_dir.join("start.sh"), Permissions::from_mode(0o777))?;
        fs::set_permissions(target_dir.join("stop.sh"), Permissions::from_mode(0o777))?;
    }

    Ok(())
}

fn update(context: &mut Context) -> anyhow::Result<()> {
    let scope = ContextUpdateAction::compose("Which value do you want to adjust")?;

    match scope {
        ContextUpdateAction::CommunicationMethod => {
            let communication = Communication::compose("Switch to which communication method")?;
            context.communication = communication;
        }
        ContextUpdateAction::RepositoryKind => {
            let repo = Repo::compose("Switch to which repository")?;
            if let Repo::Postgres = &repo {
                let postgres = PostgresContext::compose("Specify postgres configuration")?;
                context.postgres = postgres;
            }
            context.repo = repo;
        }
    }

    Ok(())
}

fn review(context: &Context) -> anyhow::Result<()> {
    let application: Application = Application::compose("Choose component")?;

    let res: anyhow::Result<Box<dyn erased_serde::Serialize>> = try {
        match application {
            Application::Api => {
                Box::new(ApiSettings::from_context(context)?) as Box<dyn erased_serde::Serialize>
            }
            Application::CommandService => Box::new(CommandServiceSettings::from_context(context)?),
            Application::EdgeRegistry => Box::new(EdgeRegistrySettings::from_context(context)?),
            Application::MaterializerOndemand => {
                Box::new(MaterializerOndemandSettings::from_context(context)?)
            }
            Application::PartialUpdateEngine => {
                Box::new(PartialUpdateEngineSettings::from_context(context)?)
            }
            Application::QueryService => Box::new(QueryServiceSettings::from_context(context)?),
            Application::DataRouter => Box::new(DataRouterSettings::from_context(context)?),
            Application::MaterializerGeneral => {
                Box::new(MaterializerGeneralSettings::from_context(context)?)
            }
            Application::ObjectBuilder => Box::new(ObjectBuilderSettings::from_context(context)?),
            Application::QueryRouter => Box::new(QueryRouterSettings::from_context(context)?),
            Application::QueryServiceTs => Box::new(QueryServiceTsSettings::from_context(context)?),
            Application::SchemaRegistry => Box::new(SchemaRegistrySettings::from_context(context)?),
        }
    };

    match res {
        Ok(setting) => println!("{}", toml::to_string(&setting)?),
        Err(error) => eprintln!("Couldn't generate config\nError: {}", error),
    }

    Ok(())
}
