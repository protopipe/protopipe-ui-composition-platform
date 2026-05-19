# 08 – Cross-cutting Concepts

## Render Function Artifacts (RFAs) and Interactive Function Artifacts (IFAs)

See [ADR-0003](../adr/0003-introduce-render-function-artifacts-as-the-rendering-boundary.md) and [ADR-0013](../adr/0013-distinguish-render-and-interactive-ui-artifacts.md).

### Render Function Artifacts (RFAs)

**Pure rendering contract:**

```text
render(data, context) → html (+meta)
```

- Deterministic output given the same inputs
- No side effects except emission of observability metrics
- Technology-agnostic internal implementation
- Teams can choose their own frontend technology
- Executed server-side by the Composer

**RFAs MUST NOT:**
- Perform experiment assignment
- Contain branching logic based on experiment variants
- Depend on global shared state
- Handle interaction logic
- Emit business events

### Interactive Function Artifacts (IFAs)

Extend RFAs with client-side interaction capabilities (see [ADR-0013](../adr/0013-distinguish-render-and-interactive-ui-artifacts.md)).

- Receive deterministic initial state from server
- React to interaction and business events
- MAY consume interaction and business events
- MUST NOT emit business events

## Message Bridge and Event-Based Interaction Model

See [ADR-0011](../adr/0011-use-event-based-interaction-model-for-ui-artifacts.md), [ADR-0012](../adr/0012-define-interaction-event-lifecyle.md), [ADR-0014](../adr/0014-use-buffered-polling-based-event-delivery.md), [ADR-0015](../adr/0015-separate-experiment-assignment-from-interaction-channel-selection.md), [ADR-0016](../adr/0016-define-message-bridge-instances-and-chaining.md), [ADR-0017](../adr/0017-use-buffered-http-batch-delivery-for-message-bridges.md), [ADR-0018](../adr/0018-use-internal-brokers-and-governed-business-event-sourcing.md), and [ADR-0019](../adr/0019-use-state-carrying-business-events-for-shadow-and-experiment-services.md).

### Event Lifecycle

Interaction events follow a defined lifecycle (see [ADR-0012](../adr/0012-define-interaction-event-lifecyle.md)):

```text
Observed → Buffered → Delivered → Processed → Resolved
```

- **Observed**: Event generated in the UI (user interaction)
- **Buffered**: Event stored locally (e.g., offline, batching)
- **Delivered**: Event sent to backend (transport acknowledged)
- **Processed**: Backend validates and processes (results in business event or rejection)
- **Resolved**: UI receives result and reconciles optimistic state

### Event Types

- **Interaction Events**: Emitted by UI artifacts (represent user interaction or UI intent)
- **Response Events**: Emitted by services or bridges to resolve a previous interaction through a correlation identifier
- **Business Events**: Emitted by backend services (represent validated business state)
- **Technical Events**: Emitted by infrastructure (transport, validation, failure)

### Message Bridge Profiles

- **Client-side Message Bridge**: Runs in the browser or PWA, buffers observed interaction events locally when needed, propagates authentication context, synchronizes batches upstream, and reconciles response events.
- **Server-side Message Bridge**: Acts as policy gate, durable delivery endpoint, channel resolver, and internal integration adapter.
- **Edge Message Bridge**: Buffers and forwards events for local networks, field usage, constrained connectivity, or bridge chaining.

### Delivery Mechanism

Buffered, polling-based event delivery (see [ADR-0014](../adr/0014-use-buffered-polling-based-event-delivery.md)):

- Events flow through client-side and server-side message bridges
- Clients retrieve pending events via polling (`GET /messages`)
- Push sockets are not required for frontend-backend communication
- Supports offline clients and deferred delivery
- The Composer remains authoritative for experiment assignment
- The Message Bridge selects configured publish/consume channels from effective Page and Experiment configuration
- Service and bridge consumers receive buffered HTTP batches by default
- Batches are committed only after acknowledgement; delivery is at-least-once
- Consumers may request cursor-based replay when the stream supports retention or sourcing
- Durable brokers such as RabbitMQ provide internal buffering, retry, dead-lettering, and outbox/inbox processing
- Kafka or compatible event-log technology may provide optional sourcing for configured business-event streams

### Governed Business Events

- Business events use typed schemas.
- Every payload property must declare its sensitivity and disclosure rules.
- State-changing business events carry version information.
- Business events carry the state required by their intended consumers; consumers do not depend on follow-up domain-service reads to interpret the event.
- Consumers are registered with trust level, scopes, batch limits, and replay permissions.
- The Message Bridge performs field-level redaction during delivery.
- Events are not removed from a consumer sequence only because payload fields are hidden; the cursor and required envelope metadata remain visible.
- If a consumer may not read any payload fields, the event is delivered with an empty payload object.
- Shadow and experiment services synchronize from business-object event streams before they are activated for traffic.

## Experimentation

See [ADR-0004](../adr/0004-use-artifact-based-experiments-instead-of-feature-toggles-in-product-code.md) and [ADR-0008](../adr/0008-centralize-experiment-routing-before-rendering.md).

- Experiments route to different artifact implementations
- No feature toggles inside product code
- Snapshot artifacts are pinned and promoted without modifying merged code
- Experiment assignment is centralized and happens before rendering (see [ADR-0008](../adr/0008-centralize-experiment-routing-before-rendering.md))
- If no consent is given: assign default (control) variant (see [ADR-0009](../adr/0009-use-default-variant-when-no-consent-is-given.md))

## Telemetry: Observability vs Analytics

See [ADR-0005](../adr/0005-keep-metrics-in-prometheus-avoid-counters-in-crd-status.md) and [ADR-0010](../adr/0010-seperate-observability-from-experiment-analytics.md).

### Observability

- Collected for operational and technical purposes only
- Includes: request counts, response times, error rates
- Storage: Prometheus metrics, short-lived aggregates
- MUST NOT include persistent user identifiers

### Experiment Analytics

- Used to evaluate experiments and measure KPIs
- MUST only be collected if consent is given (see [ADR-0009](../adr/0009-use-default-variant-when-no-consent-is-given.md))
- Separate pipeline from observability
- User-related data must be deletable upon request
- Anonymization must be irreversible

## Kubernetes-Native Integration

See [ADR-0002](../adr/0002-model-pages-and-experiments-as-kubernetes-crds.md).

- Pages and Experiments are modeled as Kubernetes CRDs
- Declarative configuration managed via GitOps workflows
- Kubernetes operator reconciles resources

## Testability and Shift-left

- CDCTs validate frontend-backend contracts.
- Storybook serves as integration and documentation hub.
- RFAs are executable locally with verified mock data.
- Snapshot artifacts enable production-near testing

## Security and Isolation

- RFAs are isolated at execution level.
- Limited context exposure.
- No shared global state between artifacts.
- Strong separation between components powered by event contracts
