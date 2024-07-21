pub use config::ConfigError;

use crate::discord;
use crate::moonraker;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub discord: discord::Config,
    pub moonraker: moonraker::Config,
}

pub fn load() -> Result<Config, ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::File::with_name("config.local"))
        .build()?
        .try_deserialize::<Config>()
}
