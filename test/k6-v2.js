import http from 'k6/http';
import { check } from 'k6';

// --env TARGET=nginx|haproxy|envoy|kong|all  (default: all)
const TARGET = __ENV.TARGET || 'all';

const TARGETS = {
  nginx:   ['http://192.168.100.81:80'],
  haproxy: ['http://192.168.100.80:80'],
  envoy:   ['http://192.168.100.82:80'],
  kong:    ['http://192.168.100.83:80'],
  all:     Array.from({ length: 10 }, (_, i) => `http://192.168.100.${60 + i}:8123`),
};

const urls = TARGETS[TARGET];
if (!urls) {
  throw new Error(`Unknown TARGET "${TARGET}". Use nginx, haproxy, envoy, kong, or all.`);
}

export const options = {
  scenarios: {
    find_edge: {
      executor: 'ramping-arrival-rate',
      startRate: 100,
      timeUnit: '1s',
      stages: [
        { duration: '30s', target:  1000 },
        { duration: '30s', target:  2000 },
        { duration: '30s', target:  4000 },
        { duration: '30s', target:  8000 },
        { duration: '30s', target: 16000 },
        { duration: '30s', target: 32000 },
      ],
      preAllocatedVUs: 500,
      maxVUs: 8000,
    },
  },
  thresholds: {
    http_req_duration: [{ threshold: 'p(95)<2000', abortOnFail: false }],
    http_req_failed:   [{ threshold: 'rate<0.20',  abortOnFail: false }],
  },
};

export default function () {
  const url = urls[Math.floor(Math.random() * urls.length)];
  const res = http.get(url);
  check(res, { 'status 200': (r) => r.status === 200 });
}
