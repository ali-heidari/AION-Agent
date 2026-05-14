import http from "k6/http";
import { check, sleep } from "k6";
import { scenario } from 'k6/execution';

const TARGET_IPS = [];
for (let i = 60; i <= 69; i++) {
    TARGET_IPS.push(`http://192.168.1.${i}:8123`);
}

export const options = {
    scenarios: {
        stress_test: {
            executor: 'ramping-vus',
            stages: [
                { duration: "30s", target: 100 }, // ramp up — all 10 nodes
                { duration: "1m",  target: 300 }, // stage 1 — 7 nodes (.60–.66)
                { duration: "1m",  target: 500 }, // stage 2 — 8 nodes (.60–.67)
                { duration: "1m",  target: 800 }, // stage 3 — 2 nodes (.60–.61) extreme
                { duration: "1m",  target: 500 }, // stage 4 — 5 nodes (.60–.64)
                { duration: "30s", target: 0   }, // ramp down — all 10 nodes
            ],
            startVUs: 0,
            gracefulStop: "5s",
        },
    },
    thresholds: {
        http_req_duration: ["p(99) < 3000"],
    },
};

function getCurrentTargetGroup(timeOffset) {
    if (timeOffset < 30)  return TARGET_IPS.slice(0, 10); // all 10
    if (timeOffset < 90)  return TARGET_IPS.slice(0, 7);  // 7 nodes
    if (timeOffset < 150) return TARGET_IPS.slice(0, 8);  // 8 nodes
    if (timeOffset < 210) return TARGET_IPS.slice(0, 2);  // 2 nodes — extreme
    if (timeOffset < 270) return TARGET_IPS.slice(0, 5);  // 5 nodes
    return TARGET_IPS.slice(0, 10);                        // ramp down
}

export default function () {
    const timeOffset = (Date.now() - scenario.startTime) / 1000;
    const targetGroup = getCurrentTargetGroup(timeOffset);
    const targetUrl = targetGroup[Math.floor(Math.random() * targetGroup.length)];

    const res = http.get(targetUrl);

    check(res, {
        "status 200": (r) => r.status === 200,
        [`active nodes: ${targetGroup.length}`]: true,
    });

    sleep(1);
}
