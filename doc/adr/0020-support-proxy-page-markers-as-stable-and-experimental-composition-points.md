# 0020 - Support Proxy Page Markers as Stable and Experimental Composition Points

Date: 2026-06-23

## Status

Accepted

## Context

The platform must support a "day 1" migration path for decomposing an existing
frontend monolith into independently owned frontend services.

In this migration mode, the Composer is placed between ingress and the legacy
monolith. The monolith continues to render the surrounding page while selected
regions are marked with inert HTML comments. The Composer can then replace
marked regions with Render Function Artifacts (RFAs) without requiring the
monolith to know about experiments, artifacts, or frontend composition.

This use case is both a Page concern and an Experiment concern:

- A Page owns the stable route, upstream origin, and accepted composition
  surface for a proxy page.
- An Experiment owns temporary replacement bindings, candidate markers, and
  traffic assignment for A/B tests, canaries, and preview validation.

Existing decisions still apply:

- Pages and Experiments are modeled as Kubernetes resources (ADR-0002).
- RFAs are the rendering boundary (ADR-0003).
- Experiments use artifact routing instead of product-code feature toggles
  (ADR-0004).
- Rendering remains SSR-first (ADR-0007).
- Experiment assignment is centralized before rendering (ADR-0008).

## Decision

The platform supports **Proxy Pages** as a Page delivery mode.

A Proxy Page forwards a request to an upstream HTTP origin and streams the
upstream response back to the client. If the effective Page configuration
contains proxy marker replacements, the Composer scans the upstream HTML stream
for configured marker regions and replaces matching regions with rendered RFAs.

Proxy marker replacements can come from two places:

- **Accepted marker replacements** in the Page definition. These are stable
  bindings that have been promoted out of experiments and are part of normal
  page delivery.
- **Candidate marker replacements** in Experiment variants. These are temporary
  bindings used for A/B tests, canaries, preview validation, and migration
  experiments.

The effective Proxy Page configuration is produced by applying Experiment
overrides to the Page configuration after centralized assignment and before
rendering starts.

Proxy markers are HTML comment based composition points. A marker has a stable
identifier and a bounded region, for example:

```html
<!-- protopipe:marker checkout.summary -->
<div>Legacy checkout summary</div>
<!-- /protopipe:marker checkout.summary -->
```

If no replacement is active for a marker, the upstream content passes through
unchanged. This allows the monolith to introduce inert markers safely before any
traffic is assigned to Composer-native replacements.

## Streaming Runtime Requirements

Proxy Pages MUST be streamed.

The Composer MUST start forwarding upstream response bytes as soon as possible
while preserving the ability to replace configured marker regions.

For active marker replacements, the Composer SHOULD start data loading and RFA
rendering in parallel with upstream streaming as soon as the effective Page
configuration is known. If upstream streaming reaches a marker before the
replacement is ready, the Composer MAY block at that marker until the configured
replacement result, timeout, or fallback decision is available.

The user MUST NOT receive partial marker fallback content followed by a late
replacement for the same marker. A marker region is delivered either as upstream
fallback content or as the replacement RFA output.

Timeout and failure handling MUST be explicit per marker replacement. The
default migration-safe behavior is to keep the upstream marker content when a
replacement cannot be rendered in time.

## Runtime Flow

The canonical Proxy Page rendering flow is:

```text
Request
 -> Page Resolution
 -> Experiment Assignment
 -> Effective Proxy Page Configuration
 -> Upstream Request
 -> Data Loading and RFA Rendering for Active Markers (parallel)
 -> Streaming Marker Detection
 -> Marker Replacement or Fallback
 -> Streamed HTML Response
 -> Telemetry Emission
```

Experiment assignment still happens before rendering. RFAs still receive
explicit data and context and MUST NOT perform experiment assignment.

## Consequences

### Positive

- Existing monolith pages can be decomposed incrementally.
- Initial markers can be introduced without changing user-visible behavior.
- A/B and canary tests can activate candidate marker replacements through
  Experiments.
- Successful replacements can be promoted into stable Page configuration.
- SSR-first rendering and centralized experiment routing remain intact.

### Negative / Risks

- The Composer becomes an HTTP proxy for these Pages and must handle upstream
  headers, status codes, streaming, timeouts, and cancellation carefully.
- Marker parsing introduces a streaming state machine into the runtime.
- Blocking at marker boundaries can affect tail latency if RFA rendering or
  data loading is slow.
- Incorrect marker configuration can replace the wrong upstream region.

### Mitigations

- Keep Proxy Page configuration explicit and declarative.
- Treat marker identifiers as stable composition contracts.
- Use fallback-to-upstream as the default failure behavior during migration.
- Add BDD coverage for Proxy Pages without markers, experiment-driven proxy
  activation, and marker replacement.
- Measure upstream latency, marker wait time, RFA latency, fallback count, and
  replacement count separately from experiment analytics.
