use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use dotenv::dotenv;
use lazy_static::lazy_static;
use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u32,
    pub postgres_url: String,
    pub enable_bunyan: bool,
}

lazy_static! {
    pub static ref CFG: Config =
        Config::from_env().expect("Failed to load config from environment");
}

impl Config {
    #[instrument]
    pub fn from_env() -> Result<Config> {
        dotenv().ok();

        let mut conf = config::Config::new();
        conf.merge(config::Environment::default())?;
        conf.try_into().context("Loading environment variables")
    }
}
