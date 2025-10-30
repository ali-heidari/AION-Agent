mod configurations;
mod mock;
mod reward;

use aion_rlt::CONFIG;
use aion_rlt::node::Node;
use anyhow::{Ok, Result};
use std::env;
use std::sync::{Arc, Mutex};
use std::u32;

use crate::configurations::load_config;
use crate::mock::{DatasetMode, SyntheticState};
use crate::reward::compute_reward_with_success;

fn get_features(state: &mut SyntheticState, lowest_state: u32) -> Vec<f32> {
    let features = state.next(0.0, 0.0, lowest_state);
    features
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Ok(());
    }
    let mode = &args[1];

    let mode = match mode.as_str() {
        "metrics" => DatasetMode::SystemMetrics,
        "generative" => DatasetMode::Generative,
        "csv" => DatasetMode::Inputs,
        _ => DatasetMode::Generative,
    };

    let mut dataset = SyntheticState::new(mode.clone());
    if mode == DatasetMode::Inputs {
        dataset.from_csv_dataset().ok();
    }
    let state = Arc::new(Mutex::new(dataset));
    let cloned_state = Arc::clone(&state);

    aion_rlt::initialize(load_config().unwrap());

    Node::start(
        move |lowest_state| get_features(&mut cloned_state.lock().unwrap(), lowest_state),
        |x, y| compute_reward_with_success(x, y as u8),
        CONFIG.get().unwrap().mode,
    )
    .await;

    Ok(())
}
