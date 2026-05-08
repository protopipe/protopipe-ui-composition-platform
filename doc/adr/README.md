# Architecture Decision Records (ADR)

This directory contains the **Architecture Decision Records (ADRs)** for the
**Protopipe Frontend Platform**.

ADRs document significant architectural decisions, including their context,
the decision taken, and the resulting consequences. They serve as a durable,
reviewable record of architectural intent and rationale.

The ADRs in this repository are maintained using the
[`adr-tools`](https://github.com/npryce/adr-tools) workflow and are versioned
together with the source code.

---

## Accepted Decisions

- [ADR-0001: Record Architecture Decisions](0001-record-architecture-decisions.md)
- [ADR-0002: Model Pages and Experiments as Kubernetes CRDs](0002-model-pages-and-experiments-as-kubernetes-crds.md)
- [ADR-0003: Introduce Render Function Artifacts as the Rendering Boundary](0003-introduce-render-function-artifacts-as-the-rendering-boundary.md)
- [ADR-0004: Use Artifact-based Experiments Instead of Feature Toggles in Product Code](0004-use-artifact-based-experiments-instead-of-feature-toggles-in-product-code.md)
- [ADR-0005: Keep Metrics in Prometheus; Avoid Counters in CRD Status](0005-keep-metrics-in-prometheus-avoid-counters-in-crd-status.md)
- [ADR-0006: Adopt Docs-as-Code with arc42 Chapters and PlantUML Diagrams](0006-adopt-docs-as-code-with-arc42-chapters-and-plantuml-diagrams.md)
- [ADR-0007: SSR-first Rendering Model](0007-ssr-first-rendering-model.md)
- [ADR-0008: Centralize Experiment Routing Before Rendering](0008-centralize-experiment-routing-before-rendering.md)
- [ADR-0009: Use Default Variant When No Consent is Given](0009-use-default-variant-when-no-consent-is-given.md)
- [ADR-0010: Separate Observability from Experiment Analytics](0010-seperate-observability-from-experiment-analytics.md)
- [ADR-0011: Use Event-Based Interaction Model for UI Artifacts](0011-use-event-based-interaction-model-for-ui-artifacts.md)
- [ADR-0012: Define Interaction Event Lifecycle](0012-define-interaction-event-lifecyle.md)
- [ADR-0013: Distinguish Render and Interactive UI Artifacts](0013-distinguish-render-and-interactive-ui-artifacts.md)
- [ADR-0014: Use Buffered Polling-Based Event Delivery](0014-use-buffered-polling-based-event-delivery.md)

---

## Proposed Decisions

_(none at this time)_

---

## Superseded Decisions

_(none at this time)_

---

## ADR Lifecycle

Each ADR follows a simple lifecycle:

1. **Proposed**  
   The decision is under discussion and not yet binding.

2. **Accepted**  
   The decision is agreed upon and considered architectural guidance.

3. **Superseded**  
   The decision has been replaced by a newer ADR, which must explicitly
   reference the superseded decision.

---

## Contribution Guidelines

- Architectural changes of structural relevance **require an ADR**.
- ADRs must follow the standard template:
  - Context
  - Decision
  - Consequences
- ADRs are reviewed by the core maintainers.
- Once accepted, ADRs should not be modified retroactively; changes require
  a new ADR that supersedes the previous one.

---

## Relationship to arc42

The arc42 chapter  
**09 – Architecture Decisions**  
references this directory as the authoritative source for architectural
decisions.

See:  
`docs/arc42/09-architecture-decisions.md`

