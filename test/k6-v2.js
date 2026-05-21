import http from 'k6/http';
import { sleep, check } from 'k6';

// --env TARGET=nginx|haproxy|all  (default: all)
const TARGET = __ENV.TARGET || 'all';

const TARGETS = {
  nginx:   ['http://192.168.100.81:80'],
  haproxy: ['http://192.168.100.80:80'],
  all:     Array.from({ length: 10 }, (_, i) => `http://192.168.100.${60 + i}:8123`),
};

const urls = TARGETS[TARGET];
if (!urls) {
  throw new Error(`Unknown TARGET "${TARGET}". Use nginx, haproxy, or all.`);
}

export const options = {
  stages: [
    { duration: '30s', target: 50 },   // ramp up
    { duration: '1m',  target: 100 },  // normal load
    { duration: '30s', target: 200 },  // stress — forces agent to state 0
    { duration: '30s', target: 100 },  // recovery — watch agent return to state 1
    { duration: '30s', target: 0 },    // ramp down
  ],
  thresholds: {
    http_req_duration: ['p(99)<50'],  // sub-50ms p99
    http_req_failed:   ['rate<0.01'], // less than 1% errors
  },
};

export default function () {
  const url = urls[Math.floor(Math.random() * urls.length)];
  const res = http.get(url);
  check(res, { 'status 200': (r) => r.status === 200 });
  sleep(0.1);
}
