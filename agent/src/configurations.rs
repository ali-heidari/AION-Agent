use std::sync::Arc;

use aixker_rlt::configurations::Configurations;
use aixker_rlt::node::RunningMode;
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    pub interval_secs: u64,
    pub batch_size: u32,
    pub total_batches: usize,
    pub input_number: usize,
    pub output_number: usize,
    pub hidden_layers: usize,
    pub reply_capacity: usize,
    pub model_name: String,
    pub log_interval: u64,
    pub mode: RunningMode,
    #[serde(default)]
    pub busy: bool,
    #[serde(default)]
    pub neighbor: String,
}

impl From<&AgentConfig> for Configurations {
    fn from(c: &AgentConfig) -> Self {
        Configurations {
            interval_secs: c.interval_secs,
            batch_size: c.batch_size,
            total_batches: c.total_batches,
            input_number: c.input_number,
            output_number: c.output_number,
            hidden_layers: c.hidden_layers,
            reply_capacity: c.reply_capacity,
            model_name: c.model_name.clone(),
            log_interval: c.log_interval,
            mode: c.mode,
        }
    }
}

pub fn load_config() -> Result<Arc<AgentConfig>, config::ConfigError> {
    let settings = Config::builder()
        .add_source(File::with_name("config.toml"))
        .add_source(Environment::with_prefix("APP").separator("__"))
        .build()?;
    Ok(Arc::new(settings.try_deserialize()?))
}
