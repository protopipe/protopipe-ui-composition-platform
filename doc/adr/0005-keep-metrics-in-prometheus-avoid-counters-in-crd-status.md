# 0005 – Keep Metrics in Prometheus; Avoid Counters in CRD Status

Date: 2025-12-29

## Status

Accepted

## Context

Kubernetes CRD status fields are designed to represent reconciliation state and
conditions, not high-frequency or time-series metrics.

Embedding counters or metrics into CRD status would increase API server load,
complicate reconciliation logic, and blur responsibility boundaries.

## Decision

Operational metrics such as request counts, cache hits, and latencies are
exported exclusively via **Prometheus metrics**.

CRD status fields are limited to:
- Effective configuration
- Resolution results
- Conditions and error states

## Consequences

- Clear separation between control-plane state and telemetry.
- Better scalability and observability.
- Metrics analysis relies on external tooling (Prometheus/Grafana).
- CRD status remains stable and predictable.

