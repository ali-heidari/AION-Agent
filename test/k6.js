import http from "k6/http";
import { check, sleep } from "k6";
import { scenario } from 'k6/execution';

// 1. Define the 10 target IP addresses (simulated 10 IPs from 60 to 70)
// NOTE: Replace these with your actual IP addresses or domain names!
const TARGET_IPS = [];
for (let i = 60; i <= 69; i++) {
    // Assuming your targets are HTTP/S endpoints, use the correct protocol
    TARGET_IPS.push(`http://192.168.1.${i}:8123`); 
}

// 2. Define the phases and the IPs targeted in each phase
// The 'phases' array defines the overall VU load profile (the 'underpressure' part).
// The 'loadProfile' defines WHICH IPs are hit during those phases.
// The total duration is 1m + 1m + 1m + 1m = 4 minutes (excluding ramp-ups/downs).
export const options = {
    // Use the 'ramping-vus' executor to define the pressure stages
    scenarios: {
        stress_test: {
            executor: 'ramping-vus',
            // Total VUs will ramp up to 300, 500, etc.
            stages: [
                // 1. Ramp up to 100 VUs over 30s (initial warm-up/pressure)
                { duration: "30s", target: 100 },
                
                // 2. STAGE 1 Pressure: Hit IPs 60-69 (all 10)
                { duration: "1m", target: 300 }, // Apply high pressure
                
                // 3. STAGE 2 Pressure: Hit 9 IPs (60-68)
                { duration: "1m", target: 500 }, // Apply higher pressure
                
                // 4. STAGE 3 Pressure: Hit 2 IPs (60-61)
                { duration: "1m", target: 800 }, // Apply extreme pressure
                
                // 5. STAGE 4 Pressure: Hit 5 IPs (60-64)
                { duration: "1m", target: 500 }, // Back to high pressure
                
                // 6. Ramp down to 0 VUs over 30s
                { duration: "30s", target: 0 },
            ],
            startVUs: 0,
            gracefulStop: "5s",
        },
    },
    // Assert that 99% of requests finish within 3000ms.
    thresholds: {
        http_req_duration: ["p(99) < 3000"],
    },
};

// 3. Function to determine the target IP based on the current scenario time/stage
function getCurrentTargetGroup(timeOffset) {
    // The timeOffset is the time elapsed since the scenario started (in seconds)
    // We use the same durations as defined in the 'stages' array above.
    // Note: The ramp-up/down periods must be accounted for.
    
    // Ramp up ends at 30s.
    if (timeOffset < 30) {
        // During ramp-up, hit all 10
        return TARGET_IPS.slice(0, 10);
    } 
    // STAGE 1 (1m duration) is from 30s to 90s (30 + 60)
    else if (timeOffset < 90) {
        // Hit 10 IPs: indices 0-9
        return TARGET_IPS.slice(0, 7); 
    } 
    // STAGE 2 (1m duration) is from 90s to 150s (90 + 60)
    else if (timeOffset < 150) {
        // Hit 9 IPs: indices 0-8
        return TARGET_IPS.slice(0, 8);
    } 
    // STAGE 3 (1m duration) is from 150s to 210s (150 + 60)
    else if (timeOffset < 210) {
        // Hit 2 IPs: indices 0-1
        return TARGET_IPS.slice(0, 2);
    } 
    // STAGE 4 (1m duration) is from 210s to 270s (210 + 60)
    else if (timeOffset < 270) {
        // Hit 5 IPs: indices 0-4
        return TARGET_IPS.slice(0, 5);
    } 
    // Ramp down starts at 270s
    else {
        // During ramp-down, hit all 10
        return TARGET_IPS.slice(0, 10);
    }
}

// 4. Simulated user behavior
export default function () {
    const timeOffset = scenario.progress * scenario.iterationDuration / 1000;
    const targetGroup = getCurrentTargetGroup(timeOffset);
    
    // Choose one IP randomly from the currently active target group
    const targetUrl = targetGroup[Math.floor(Math.random() * targetGroup.length)];

    let res = http.get(targetUrl);
    
    // Validate response status
    check(res, {
        "is status 200": (r) => r.status == 200,
        // Optional: Tag the request with the current stage for analytics
        [`stage_target_ips: ${targetGroup.length}`]: true,
    });
    
    sleep(1);
}