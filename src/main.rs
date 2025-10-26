mod configurations;
mod mock;
mod reward;

use aion_math::math::Math;
use aion_rlt::node::{Node, RunningMode};
use anyhow::Result;
use std::ops::Div;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::u32;

use crate::configurations::load_config;
use crate::mock::SyntheticState;
use crate::reward::{compute_reward, compute_reward_with_success};

fn get_features(state: &mut SyntheticState, lowest_state: u32) -> Vec<f32> {
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

    let mut c = 0;
    let mut a = vec![0; 3];
    let mut r = vec![0.0; 3];
    loop {
        // break;
        let features = get_features(&mut cloned_state.lock().unwrap(), u32::MAX);
        let raw_reward = compute_reward(&features);

        let reward = raw_reward; //((raw_reward + 1.0) / (2.0)).clamp(0.0, 1.0);
        let state = if features[0] > 0.66 && features[1] > 0.66 {
            0
        } else if features[0] > 0.33 && features[1] > 0.33 {
            1
        } else {
            2
        };
        a[state] += 1;
        r[state] = (r[state] + reward) / a[state] as f32;
        let v = Math::variance_of_ratios(a.iter().map(|x| *x as f32).collect());
        // println!("[{:?}], {}, {}, {}", features, reward, state, v);
        c += 1;
        if c > 1000000 {
            let sum = a.iter().sum::<i32>() as f32;
            let x: Vec<f32> = a.iter().map(|x| *x as f32 / sum).collect();
            println!("#{} => {:?} [{:?}] ({}) [{:?}]", c, a, x, v, r);
            break;
        }
    }
    return  Ok(());

    aion_rlt::initialize(load_config().unwrap());
    Node::start(
        move |lowest_state| get_features(&mut cloned_state.lock().unwrap(), lowest_state),
        RunningMode::Training,
        |x, y| compute_reward_with_success(x, y as u8),
    )
    .await;

    Ok(())
}
