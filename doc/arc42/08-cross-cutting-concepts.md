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

## Event-Based Interaction Model

See [ADR-0011](../adr/0011-use-event-based-interaction-model-for-ui-artifacts.md), [ADR-0012](../adr/0012-define-interaction-event-lifecyle.md) and [ADR-0014](../adr/0014-use-buffered-polling-based-event-delivery.md).

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
- **Business Events**: Emitted by backend services (represent validated business state)
- **Technical Events**: Emitted by infrastructure (transport, validation, failure)

### Delivery Mechanism

Buffered, polling-based event delivery (see [ADR-0014](../adr/0014-use-buffered-polling-based-event-delivery.md)):

- Events flow through client-side and server-side message bridges
- Clients retrieve pending events via polling (`GET /messages`)
- Push sockets are not required for frontend-backend communication
- Supports offline clients and deferred delivery

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

