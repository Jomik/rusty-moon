pub use config::ConfigError;

use crate::moonraker;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub moonraker: moonraker::Config,
}

pub fn load() -> Result<Config, ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize::<Config>()
}
