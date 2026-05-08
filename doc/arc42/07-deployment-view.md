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
- Receives requests from users or edge caches
- Emits observability and experiment analytics (separated per [ADR-0010](../adr/0010-seperate-observability-from-experiment-analytics.md))

### Message Bridge (optional)

- Handles event-based interaction between frontend and backend (see [ADR-0014](../adr/0014-use-buffered-polling-based-event-delivery.md))
- Buffers and delivers interaction events via polling
- Routes events to business services
- Stateless message routing

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

## Observability Stack

- **Metrics**: Exported to Prometheus (see [ADR-0005](../adr/0005-keep-metrics-in-prometheus-avoid-counters-in-crd-status.md))
- **Traces**: Exported to tracing infrastructure (optional)
- **Analytics**: Separate pipeline for experiment analytics (see [ADR-0010](../adr/0010-seperate-observability-from-experiment-analytics.md))

## Scaling

- Composer: Horizontally scalable (stateless)
- Operator: Single instance (leader election for HA)
- Message Bridge: Horizontally scalable (stateless)
- RFAs: Executed on-demand, isolated per request or batch

