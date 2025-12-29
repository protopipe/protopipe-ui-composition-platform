# 0002 – Model Pages and Experiments as Kubernetes CRDs

Date: 2025-12-29

## Status

Accepted

## Context

The frontend platform is Kubernetes-native and must integrate cleanly into
cluster-level workflows, GitOps processes, and declarative configuration
management.

Pages and experiments are core architectural concepts that influence routing,
rendering, and delivery behavior. Treating them as application-internal
configuration would limit observability, automation, and governance.

## Decision

Pages and Experiments are modeled as **custom Kubernetes resources (CRDs)**.

- A **Page** CRD represents the smallest deployable frontend unit.
- An **Experiment** CRD defines assignment rules and artifact overrides.

A Kubernetes Operator reconciles these resources and produces effective runtime
configuration for the frontend composer.

## Consequences

- Frontend behavior becomes declarative and cluster-visible.
- Pages and experiments integrate naturally with GitOps workflows.
- Kubernetes becomes the single source of truth for delivery configuration.
- Operator complexity increases and must be carefully managed and tested.

