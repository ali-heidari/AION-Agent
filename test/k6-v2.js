import http from 'k6/http';
import { sleep, check } from 'k6';

const TARGET_IPS = [];
for (let i = 60; i <= 69; i++) {
    TARGET_IPS.push(`http://192.168.1.${i}:8123`);
}

export const options = {
  stages: [
    { duration: '30s', target: 50 },   // ramp up
    { duration: '1m', target: 200 },   // normal load
    { duration: '30s', target: 500 },  // stress — forces agent to state 0
    { duration: '30s', target: 200 },  // recovery — watch agent return to state 1
    { duration: '30s', target: 0 },    // ramp down
  ],
  thresholds: {
    http_req_duration: ['p(99)<50'],   // your claim: sub-50ms p99
    http_req_failed: ['rate<0.01'],    // less than 1% errors
  },
};

export default function () {
  const targetUrl = TARGET_IPS[Math.floor(Math.random() * TARGET_IPS.length)];
  const res = http.get(targetUrl);
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(0.1);
}