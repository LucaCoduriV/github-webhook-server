use clap::Parser;

/// Github webhook server for CD.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Path to to the config file.
    #[arg(short, long, default_value_t = String::from("./config.toml"))]
    pub config: String,
}