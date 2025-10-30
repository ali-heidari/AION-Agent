mod configurations;
mod mock;
mod reward;

use aion_rlt::CONFIG;
use aion_rlt::node::{Node, RunningMode};
use anyhow::Result;
use std::ops::Div;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::u32;

use crate::configurations::load_config;
use crate::mock::{Mode, SyntheticState};
use crate::reward::compute_reward_with_success;

fn get_features(state: &mut SyntheticState, lowest_state: u32) -> Vec<f32> {
    let mode = match CONFIG.get().unwrap().mode {
        RunningMode::Infer => Mode::SystemMetrics,
        RunningMode::Training => Mode::Generative,
        RunningMode::TrainingWithInterval => Mode::Generative,
        _ => Mode::Inputs,
    };
    let features = state.next(mock::Mode::Generative, 0.0, 0.0, lowest_state);
    features
}

fn from_csv_dataset(state: &mut SyntheticState) -> Result<()> {
    let file_path = Path::new("vmCloud_data.csv");
    let mut rdr = csv::Reader::from_path(file_path)?;
    let max = 1.0; //if rng.gen_bool(0.04) {1.0} else {0.4};
    for result in rdr.records() {
        let record = result?;
        if record.get(2).unwrap().is_empty() || record.get(3).unwrap().is_empty() {
            continue;
        }
        let max = 1.0; //if rng.gen_bool(0.04) {1.0} else {0.4};
        let features = state.next(
            mock::Mode::Inputs,
            record
                .get(2)
                .unwrap()
                .parse::<f32>()
                .unwrap()
                .div(100.0)
                .clamp(0.05, max),
            record
                .get(3)
                .unwrap()
                .parse::<f32>()
                .unwrap()
                .div(100.0)
                .clamp(0.05, max),
            100,
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let state = Arc::new(Mutex::new(SyntheticState::new()));
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
