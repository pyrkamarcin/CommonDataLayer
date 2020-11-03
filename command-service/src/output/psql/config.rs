use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct PostgresOutputConfig {
    #[structopt(long = "postgres-output-url", env = "POSTGRES_OUTPUT_URL")]
    pub url: String,
}
