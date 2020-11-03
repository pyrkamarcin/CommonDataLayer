#![feature(box_syntax)]

use log::{error, info, trace};
use structopt::StructOpt;

use command_service::args::Args;
use command_service::communication::MessageRouter;
use command_service::input::KafkaInput;
use command_service::output::Output;
use command_service::report::ReportService;
use utils::{metrics, status_endpoints};

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::from_args();

    trace!("Environment: {:?}", args);

    let report_service = match ReportService::new(args.report_config) {
        Ok(report_service) => report_service,
        Err(error) => {
            error!("Failed to initialize report service `{}`", error);
            return;
        }
    };

    let output_service = match Output::new(args.output_config).await {
        Ok(output_service) => output_service,
        Err(error) => {
            error!("Failed to initialize output service: `{}`", error);
            return;
        }
    };

    let message_router = MessageRouter::new(
        output_service.channel(),
        report_service,
        output_service.name(),
    );

    let input_service = match KafkaInput::new(args.input_config, message_router) {
        Ok(input_service) => input_service,
        Err(error) => {
            error!("Failed to initialize input service: `{}`", error);
            return;
        }
    };

    info!("Starting the service with output {:?}", output_service);

    metrics::serve();
    status_endpoints::serve();

    tokio::select! {
        Err(input_err)   = input_service.listen() => error!("Kafka input service finished abruptly: `{:?}`", input_err),
        Err(output_err)  = output_service.run() => error!("Output service finished abruptly: `{:?}`", output_err),
        else => info!("Finished!")
    }
}
