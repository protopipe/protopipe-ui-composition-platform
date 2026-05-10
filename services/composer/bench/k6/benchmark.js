import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: __ENV.K6_VUS ? parseInt(__ENV.K6_VUS) : 5,
  duration: __ENV.K6_DURATION ? __ENV.K6_DURATION : '15s',
};

const adminBase = 'http://127.0.0.1:9000';
const renderBase = 'http://127.0.0.1:8080';
const rfaId = 'bench-rfa';
const pagePath = '/bench-page';

export function setup() {
  const rfaPayload = {
    id: rfaId,
    source: 'function render(context) { return `hello ${context.name}`; }',
    version: '1',
  };

  const pagePayload = {
    path: pagePath,
    page_id: 'bench-page',
    template: 'bench',
    rfa: rfaId,
    timeout_ms: 1000,
    data: {
      name: {
        type: 'static',
        value: 'k6',
      },
    },
  };

  let res = http.post(`${adminBase}/admin/rfa/register`, JSON.stringify(rfaPayload), {
    headers: { 'Content-Type': 'application/json' },
  });
  check(res, {
    'registered rfa': (r) => r.status === 201,
  });

  res = http.post(`${adminBase}/admin/config/pages`, JSON.stringify(pagePayload), {
    headers: { 'Content-Type': 'application/json' },
  });
  check(res, {
    'registered page': (r) => r.status === 201,
  });

  return { pagePath };
}

export default function (data) {
  const url = `${renderBase}${data.pagePath}?run=${__VU}-${__ITER}-${Math.random()}`;
  const res = http.get(url, { headers: { Accept: 'text/html' } });

  check(res, {
    'status 200': (r) => r.status === 200,
    'body contains hello': (r) => r.body.includes('hello k6'),
  });

  sleep(Math.random() * 0.2);
}
