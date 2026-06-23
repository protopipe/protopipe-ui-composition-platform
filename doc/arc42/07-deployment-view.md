# 07 – Deployment View

## Kubernetes Resources

The platform is deployed as a set of Kubernetes resources:

### Operator (Frontend Operator)

- Watches Page and Experiment CRDs (see [ADR-0002](../adr/0002-model-pages-and-experiments-as-kubernetes-crds.md))
- Reconciles resources to produce effective runtime configuration
- Updates conditions and status

### Composer Service

- Stateless HTTP service for page composition
- Executes RFAs, loads templates, resolves slots
- Streams Proxy Page upstream responses and replaces configured marker regions
  when effective Page configuration contains active proxy marker replacements
- Receives requests from users or edge caches
- Emits observability and experiment analytics (separated per [ADR-0010](../adr/0010-seperate-observability-from-experiment-analytics.md))

### Message Bridge (optional)

- Handles event-based interaction between frontend and backend (see [ADR-0014](../adr/0014-use-buffered-polling-based-event-delivery.md))
- Exists as client-side, server-side, and edge bridge profiles that share a common protocol model (see [ADR-0016](../adr/0016-define-message-bridge-instances-and-chaining.md))
- Publishes commands and interaction events via configured channels
- Delivers response events to clients via polling and delivers service/bridge batches via buffered HTTP webhooks (see [ADR-0017](../adr/0017-use-buffered-http-batch-delivery-for-message-bridges.md))
- Selects configured channels from effective Page and Experiment configuration without performing experiment assignment (see [ADR-0015](../adr/0015-separate-experiment-assignment-from-interaction-channel-selection.md))
- Uses internal brokers such as RabbitMQ for buffering, retry, dead-lettering, and outbox/inbox processing
- May use Kafka or compatible event-log technology for configured business-event sourcing (see [ADR-0018](../adr/0018-use-internal-brokers-and-governed-business-event-sourcing.md))
- Keeps public consumers independent from broker-specific protocols

### Durable Message Broker (optional)

- Provides durable queues, exchanges/topics, bindings, retries, and dead-letter queues for interaction delivery
- Supports runtime topology changes for new Pages, IFAs, and Experiments
- Acts as internal Message Bridge infrastructure rather than the public consumer contract
- May provide event-log storage for sourced business-event streams
- Performs technical message persistence and distribution; business validation remains with backend services

## Artifact Deployment

### Render Function Artifacts (RFAs)

- Published as versioned artifacts (e.g., Docker images, npm packages)
- Referenced by Pages and Experiments
- Executed server-side by the Composer
- Isolated at execution level (separate processes or VMs)

### Pages and Experiments

- Modeled as Kubernetes CRDs (see [ADR-0002](../adr/0002-model-pages-and-experiments-as-kubernetes-crds.md))
- Stored in etcd (Kubernetes API server)
- Updated via GitOps workflows
- Changes are versioned and auditable
- Proxy Pages may reference upstream origins and accepted marker replacements;
  Experiments may add candidate marker replacements to the effective runtime
  configuration (see [ADR-0020](../adr/0020-support-proxy-page-markers-as-stable-and-experimental-composition-points.md))

## Observability Stack

- **Metrics**: Exported to Prometheus (see [ADR-0005](../adr/0005-keep-metrics-in-prometheus-avoid-counters-in-crd-status.md))
- **Traces**: Exported to tracing infrastructure (optional)
- **Analytics**: Separate pipeline for experiment analytics (see [ADR-0010](../adr/0010-seperate-observability-from-experiment-analytics.md))

## Scaling

- Composer: Horizontally scalable (stateless)
- Operator: Single instance (leader election for HA)
- Message Bridge: Horizontally scalable service instances with durable broker-backed delivery state
- Durable Message Broker: Scaled and operated according to the selected broker technology
- RFAs: Executed on-demand, isolated per request or batch
