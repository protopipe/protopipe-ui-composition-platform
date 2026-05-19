# 0015 – Separate Experiment Assignment from Interaction Channel Selection

Date: 2026-05-19

## Status

Accepted

## Context

The platform distinguishes deterministic rendering from interactive behavior
(see [ADR-0013](0013-distinguish-render-and-interactive-ui-artifacts.md)) and
uses buffered polling-based event delivery for interaction flows (see
[ADR-0014](0014-use-buffered-polling-based-event-delivery.md)).

Interactive Function Artifact (IFA) pages need to publish commands or
interaction events and later receive response events. The concrete messaging
channels may differ by page, artifact, tenant, experiment, variant, or rollout
stage.

At the same time, experiment assignment is already an architectural
responsibility of the Composer and happens before rendering begins (see
[ADR-0008](0008-centralize-experiment-routing-before-rendering.md)). The
Message Bridge must therefore not become a second experiment assignment engine.

The platform also needs enterprise-grade delivery properties:

- accepted messages must not disappear silently,
- message delivery must tolerate temporary failures and offline clients,
- new pages, IFAs, and experiments must be introducible at runtime,
- broker topology must support additive runtime changes,
- command delivery, business events, response events, and technical events must
  remain distinguishable.

## Decision

Experiment assignment remains authoritative in the Composer.

The Composer:

- resolves the page, artifact, experiment, and variant before rendering,
- sets or clears experiment assignment cookies,
- renders the initial page state,
- injects the interaction client configuration for IFA pages,
- includes the render-time page/artifact/experiment context needed by the
  client-side message library.

The client-side message library communicates directly with the Message Bridge
via REST publishing and polling. Cookies and render-time context accompany
message requests.

The Message Bridge:

- does **not** create or change experiment assignments,
- may validate that the message envelope matches the authoritative assignment
  context,
- resolves configured publish and consume channels from the effective page and
  interaction configuration,
- persists accepted messages before acknowledging them,
- exposes failures as technical events or observable delivery failures.

The Message Bridge should not contain complex business routing logic. Its
primary responsibility is channel selection and durable delivery:

```text
validate envelope
resolve configured channel
publish or consume through broker
acknowledge, retry, dead-letter, or expose failure
```

The effective interaction configuration describes channels rather than embedding
business routing logic in the bridge:

```text
command / interaction event -> publish exchange/topic/routing key
response event               -> consume queue/topic/cursor
technical event              -> observe or deliver failure state
```

Broker technology, initially RabbitMQ for the Durchstich, is responsible for
technical message distribution through exchanges, topics, queues, bindings,
retry queues, and dead-letter queues.

Multiple bindings or topics may feed the same queue. Multiple services that
need the same message must use separate queues bound to the same exchange or
topic. Multiple consumers on the same queue are treated as a competing consumer
group.

Configuration ownership is declarative:

- users configure interaction as part of the Page configuration,
- experiments may override interaction channel configuration,
- the Operator later projects Page and Experiment resources into:
  - Composer runtime configuration,
  - Message Bridge runtime configuration,
  - broker topology.

Until CRDs are implemented, the same separation is represented in the JSON page
configuration used by the Durchstich.

## Consequences

### Positive

- Experiment assignment remains centralized and reproducible.
- The Message Bridge stays independent from rendering and experiment assignment.
- Runtime changes for new pages, IFAs, and experiments can be applied by
  changing effective channel configuration and broker topology.
- RabbitMQ can provide flexible technical distribution without leaking broker
  semantics into UI artifacts.
- The platform can support at-least-once delivery while leaving semantic
  idempotency and authoritative business decisions to business services.
- The client-side message library has a clear contract: publish commands or
  interaction events, poll response events, preserve correlation identifiers,
  and reconcile UI state.

### Negative / Risks

- More runtime configuration must be generated, validated, and observed.
- Misconfigured channels can route messages to the wrong service or leave them
  undeliverable.
- Experiment overrides of interaction channels can make message flows harder to
  understand without good inspection tooling.
- At-least-once delivery may create duplicate messages.
- Broker-specific topology concepts may leak into Page configuration if the
  configuration model is not kept abstract enough.

### Mitigations

- Validate effective Page and interaction configuration before activation.
- Treat broker topology changes as additive and non-destructive by default.
- Require message identifiers and correlation identifiers.
- Require business services to implement semantic idempotency where commands can
  be retried.
- Make channel resolution, publish results, retries, dead-lettering, and
  delivery lag observable.
- Keep broker-specific details behind Message Bridge and Operator adapters where
  possible.
- Provide inspection tooling that shows the effective page, experiment variant,
  channel configuration, and broker bindings for an IFA page.
