use std::sync::Arc;

use aion_rlt::configurations::Configurations;
use config::{Config, Environment, File};

pub fn load_config() -> Result<Arc<Configurations>, config::ConfigError> {
    let settings = Config::builder()
        .add_source(File::with_name("config.toml"))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?;
    let worker_config: Arc<Configurations> = Arc::new(settings.try_deserialize()?);
    Ok(worker_config)
}
