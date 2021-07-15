use color_eyre::Result;
use dotenv::dotenv;
use eyre::WrapErr;
use lazy_static::lazy_static;
use serde::Deserialize;
use tracing::{info, instrument};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u32,
    pub postgres_url: String,
}

lazy_static! {
    pub static ref CFG: Config =
        Config::from_env().expect("Failed to load config from environment");
}

impl Config {
    #[instrument]
    pub fn from_env() -> Result<Config> {
        dotenv().ok();

        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();

        info!("Loading config");

        let mut conf = config::Config::new();
        conf.merge(config::Environment::default())?;
        conf.try_into().context("Loading environment variables")
    }
}
