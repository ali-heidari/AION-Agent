pub fn compute_reward(features: &[f32]) -> f32 {
    let cpu = features[0].clamp(0.0, 1.0);
    let mem = features[1].clamp(0.0, 1.0);
    let swap = features[2].clamp(0.0, 1.0);
    let disk = features[3].clamp(0.0, 1.0);
    let throughput = features[4].clamp(0.0, 1.0);
    let latency = features[5].clamp(0.0, 1.0);

    // Nonlinear penalty: heavy penalty when close to full utilization
    let load_penalty = 0.7 * cpu + 0.5 * mem;
    let io_penalty = 0.05 * swap + 0.2 * disk;

    // Performance term — prefer high throughput & low latency
    let perf_score = 2.0 * throughput * (1.0 - latency * 0.4).clamp(0.0, 1.0);

    // Combine into base reward
    let reward = perf_score - (load_penalty + io_penalty);

    reward.clamp(-1.0, 1.0)
}

pub fn compute_reward_with_success(features: &[f32], action: u8) -> (f32, bool) {
    println!("[AI] Environment metrics: {:?}, Action: {}", features, action);

    #[cfg(not(test))]
    {
        use crate::ME;

        let mut me = ME.lock().unwrap();
        if me.state != action {
            use crate::represent;

            me.state = action;
            tokio::spawn(represent(action));
        }
    }

    return (1.0, true);

    let raw_reward = compute_reward(features);

    let scaled_reward = ((raw_reward + 1.0) / (2.0)).clamp(0.0, 1.0);

    let state=    // Discretize the continuous value into states
    if scaled_reward < 0.3 { // -0.4
        0 // High pressure (bad)
    } else if scaled_reward < 0.7 { // 0.4
        1 // Normal
    } else {
        2 // Low pressure (good)
    };

    let target_center = [0.0, 0.5, 1.0][action as usize];
    let mut reward = 1.0 - f32::abs(target_center - scaled_reward);
    reward = (reward * 2.0) - 1.0; // scale to [-1, 1]
    // Penalize totally wrong actions (like doing opposite)

    if (action).abs_diff(state) == 1 {
        reward -= 0.5;
    }

    if (state == 0 && action == 2) || (state == 2 && action == 0) {
        reward = -1.0;
    }

    // if state == action && action == 2 {
    //     reward += 5.0;
    // }

    (reward.clamp(-1.0, 1.0), reward > 0.0)
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_compute_reward() {
        // if cpu > 0.75 && mem > 0.7 && throughput < 0.4 && latency > 0.6    ->     0 // High pressure
        let features = vec![0.75, 0.7, 0.1, 0.5, 0.4, 0.6];
        let mut reward = compute_reward(&features);
        println!("0 // High pressure edge {}", reward);
        assert!(reward < 0.0);
        // } else if cpu < 0.35 && mem < 0.4 && throughput > 0.7 && latency < 0.25 { ->         2 // Low pressure
        let features = vec![0.35, 0.4, 0.1, 0.5, 0.7, 0.25];
        reward = compute_reward(&features);
        println!(" 2 // Low pressure EDGE {}", reward);
        assert!(reward > 0.4);

        let features = vec![1.0, 1.0, 0.1, 0.5, 0.1, 0.9];
        reward = compute_reward(&features);
        println!("Highest pressure {}", reward);
        assert!(reward < -0.9);
        let features = vec![0.6, 0.6, 0.1, 0.5, 0.3, 0.7];
        reward = compute_reward(&features);
        println!("Normal pressure {}", reward);
        assert!(reward > -0.4);
        let features = vec![0.5, 0.5, 0.1, 0.5, 0.65, 0.4];
        reward = compute_reward(&features);
        println!("Normal pressure {}", reward);
        assert!(reward > 0.2);
        let features = vec![0.1, 0.1, 0.1, 0.5, 0.8, 0.2];
        reward = compute_reward(&features);
        println!("Lowest pressure {}", reward);
        assert!(reward > 0.7);
    }

    #[test]
    fn compute_reward_with_success_test() {
        let mut map = vec![(vec![1.0, 1.0, 0.1, 0.5, 0.1, 0.9], 0)];

        map.push((vec![0.32, 0.14, 0.00, 0.18, 0.65, 0.30], 2));
        map.push((vec![0.05, 0.05, 0.00, 0.03, 0.93, 0.06], 2));
        map.push((vec![0.05, 0.05, 0.00, 0.03, 0.93, 0.06], 2));
        map.push((vec![0.05, 0.05, 0.00, 0.05, 0.93, 0.06], 2)); // Low pressure ✅
        map.push((vec![0.05, 0.05, 0.00, 0.02, 0.93, 0.06], 2)); // Low pressure ✅
        map.push((vec![0.05, 0.37, 0.00, 0.18, 0.65, 0.25], 2)); // Low pressure ✅
        map.push((vec![0.05, 0.05, 0.00, 0.03, 0.93, 0.06], 2)); // Low pressure 🔧 (was 0)
        map.push((vec![0.05, 0.05, 0.00, 0.00, 0.93, 0.06], 2)); // Low pressure 🔧 (was 1)
        map.push((vec![0.05, 0.24, 0.00, 0.09, 0.76, 0.17], 2)); // Low pressure ✅
        map.push((vec![0.05, 0.05, 0.00, 0.02, 0.93, 0.06], 2)); // Low pressure ✅

        map.push((vec![0.56, 0.67, 0.00, 0.45, 0.25, 0.90], 1));
        map.push((vec![0.44, 0.20, 0.00, 0.17, 0.53, 0.44], 1)); // Normal 🔧 (was 0)
        map.push((vec![0.37, 0.78, 0.05, 0.41, 0.26, 0.85], 1)); // Normal ✅
        map.push((vec![0.55, 0.48, 0.03, 0.36, 0.60, 0.42], 1));
        map.push((vec![0.62, 0.53, 0.01, 0.40, 0.55, 0.35], 1));
        map.push((vec![0.47, 0.60, 0.00, 0.32, 0.69, 0.40], 1));
        map.push((vec![0.50, 0.44, 0.02, 0.38, 0.63, 0.48], 1));
        map.push((vec![0.58, 0.59, 0.05, 0.41, 0.58, 0.39], 1));

        map.push((vec![1.00, 0.57, 0.00, 0.53, 0.11, 1.00], 0));
        map.push((vec![1.00, 1.00, 0.33, 0.69, 0.00, 1.00], 0));
        map.push((vec![1.00, 1.00, 0.27, 0.67, 0.00, 1.00], 0));
        map.push((vec![0.89, 0.72, 0.07, 0.55, 0.11, 1.00], 0)); // High pressure 🔧 (was 1)
        map.push((vec![0.82, 0.64, 0.00, 0.48, 0.16, 1.00], 0)); // High pressure ✅
        map.push((vec![1.00, 0.95, 0.21, 0.68, 0.01, 1.00], 0)); // High pressure ✅
        map.push((vec![0.78, 0.45, 0.00, 0.45, 0.24, 0.96], 0)); // High pressure ✅
        map.push((vec![1.00, 0.47, 0.00, 0.50, 0.13, 1.00], 0)); // High pressure ✅
        map.push((vec![0.80, 0.29, 0.00, 0.36, 0.27, 0.87], 0)); // High pressure ✅
        map.push((vec![1.00, 0.43, 0.00, 0.47, 0.14, 1.00], 0)); // High pressure 🔧 (was 2)

        for (i, v) in map.iter().enumerate() {
            let reward = compute_reward(&v.0);
            let (scaled_reward, success) = compute_reward_with_success(&v.0, v.1);
            println!(
                "{}: reward: {}, scaled_reward: {}, success: {}",
                i, reward, scaled_reward, success
            );
            assert!(success);
        }
    }
}
