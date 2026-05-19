import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  vus: 2,
  duration: "10s",
  thresholds: {
    "http_req_failed{phase:main}": ["rate<0.01"],
    "http_req_duration{phase:main}": ["p(95)<500"],
  },
};

const adminBase = __ENV.COMPOSER_ADMIN_URL || "http://localhost:9000";
const renderBase = __ENV.COMPOSER_BASE_URL || "http://localhost:8080";
const rfaId = "smoke-rfa";
const pagePath = "/smoke-page";

export function setup() {
  waitForComposer();

  const rfaPayload = {
    id: rfaId,
    source: "function(context) { return `hello ${context.name}`; }",
    version: "1",
  };

  const pagePayload = {
    path: pagePath,
    page_id: "smoke-page",
    type: "rfa",
    template: "smoke",
    rfa: rfaId,
    timeout_ms: 1000,
    data: {
      name: {
        type: "static",
        value: "k6",
      },
    },
  };

  http.del(`${adminBase}/admin/config`, null, {
    tags: { phase: "setup" },
    timeout: "2s",
  });

  const rfaResponse = http.post(`${adminBase}/admin/config/rfas`, JSON.stringify(rfaPayload), {
    headers: { "Content-Type": "application/json" },
    tags: { phase: "setup" },
    timeout: "2s",
  });

  if (rfaResponse.status !== 201) {
    throw new Error(`could not register RFA; status: ${rfaResponse.status}`);
  }

  const pageResponse = http.post(`${adminBase}/admin/config/pages`, JSON.stringify(pagePayload), {
    headers: { "Content-Type": "application/json" },
    tags: { phase: "setup" },
    timeout: "2s",
  });

  if (pageResponse.status !== 201) {
    throw new Error(`could not register page; status: ${pageResponse.status}`);
  }
}

export default function () {
  const response = http.get(`${renderBase}${pagePath}?run=${__VU}-${__ITER}`, {
    headers: { Accept: "text/html" },
    tags: { phase: "main" },
  });

  check(response, {
    "status is 200": (r) => r.status === 200,
    "body contains hello": (r) => r.body.includes("hello k6"),
  });
}

function waitForComposer() {
  const deadline = Date.now() + 30000;
  let lastStatus = "no response";

  while (Date.now() < deadline) {
    const response = http.get(`${adminBase}/admin/health`, {
      tags: { phase: "setup" },
      timeout: "2s",
    });

    lastStatus = `${response.status}`;

    if (response.status === 200) {
      return;
    }

    sleep(1);
  }

  throw new Error(`composer did not become ready for load test; last status: ${lastStatus}`);
}
