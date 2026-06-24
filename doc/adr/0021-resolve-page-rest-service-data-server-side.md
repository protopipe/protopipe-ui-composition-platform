# 0021 – Resolve Page REST Service Data Server-Side

Date: 2026-05-22

## Status

Accepted

## Context

ADR-0007 defines an SSR-first rendering model. ADR-0008 centralizes experiment
routing before rendering. RFAs are expected to remain deterministic render
functions that receive explicit data and context. They must not perform
experiment assignment or introduce hidden runtime dependencies.

Pages already declare structured data values such as static values, URL values,
and GET-parameter values. Some pages also need data from backend REST services
before rendering. If RFAs fetch that data themselves, the platform loses control
over:

- timeout behavior,
- retry and fallback behavior,
- observability of backend data dependencies,
- experiment-driven replacement of backend data sources,
- SSR latency budgets,
- security boundaries such as allowed outbound destinations.

REST calls are I/O-bound and must not block render workers or serialize
independent data dependencies unnecessarily.

## Decision

The Composer will support a Page data type named `restService`.

`restService` is resolved server-side by the Composer after the effective Page
configuration has been selected and after experiment overrides have been
applied, but before RFA execution starts.

The canonical flow becomes:

```text
Request
 → Page resolve
 → Experiment assignment and overrides
 → Page data resolution, including restService calls in parallel
 → RFA execution
 → HTML response
```

RFAs receive only the resolved data value in their render context. RFAs MUST NOT
receive an HTTP client, service registry handle, or direct permission to perform
backend data loading.

The first implementation scope is:

- resolve multiple `restService` entries concurrently,
- use async HTTP via Tokio-compatible clients,
- bound each call by an explicit timeout or platform default,
- allow explicit `error_default` values for degraded rendering,
- allow a small bounded retry policy,
- allow an optional fallback service declaration,
- keep the render path bounded so failed data providers cannot block a request
  indefinitely.

The Composer should prefer named services and paths over arbitrary URLs. The
`service` value references a Composer Service configuration resource. The first
resource version contains only the stable service identity and `base_url`; page
specific timeout, retry, default, and fallback settings stay in the Page data
configuration because their limits depend on the concrete rendering use case.

```json
{
  "service_id": "catalog",
  "base_url": "http://wiremock:8080/catalog"
}
```

```json
{
  "product": {
    "type": "restService",
    "service": "catalog",
    "path": "/products/{query.productId}",
    "method": "GET",
    "timeout_ms": 250,
    "retries": {
      "max_attempts": 2,
      "backoff_ms": 50
    },
    "error_default": {
      "name": "Unknown product"
    },
    "fallback": {
      "service": "catalog-cache",
      "path": "/products/{query.productId}"
    }
  }
}
```

Free-form outbound URLs are not part of the default contract. If they are
introduced for local development, they must be treated as an adapter detail and
must not weaken production governance.

Resilience features inspired by Hystrix-style systems, such as circuit breakers,
bulkheads, service health windows, and fallback-to-secondary behavior, are part
of the intended direction but not required for the first implementation slice.
They require runtime state, metrics, and clear operational semantics and will be
introduced as explicit policies.

## Consequences

### Positive

- RFAs stay deterministic and side-effect free.
- Backend data loading remains observable and governed by the Composer.
- Experiment overrides can replace data dependencies before data is loaded.
- Independent data dependencies can be resolved in parallel.
- Timeouts, retries, and fallbacks become declarative Page behavior instead of
  hidden product-code behavior.
- SSR latency can be controlled by Composer-level budgets.

### Negative / Risks

- The Composer becomes responsible for outbound service access and therefore
  needs clear security and operational controls.
- Misconfigured retries can amplify backend load.
- Fallback and default behavior can hide backend failures if observability is
  weak.
- Circuit-breaker behavior requires shared runtime state and careful rollout.

### Mitigations

- Use named services and allow-listed destinations instead of arbitrary URLs.
- Resolve named services through Composer Service configuration resources.
- Require bounded timeouts for all `restService` calls.
- Keep retry counts small and use backoff.
- Emit operational telemetry for calls, timeouts, retries, fallbacks, defaults,
  and circuit-breaker state changes.
- Treat `error_default` as an explicit product decision in Page configuration.
- Add circuit breakers and bulkheads as separate runtime policies once the basic
  `restService` data provider is stable.
