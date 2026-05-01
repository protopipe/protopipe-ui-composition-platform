# 0007 SSR-first Rendering Model

Date: 2026-04-01

## Status

Accepted

## Context

The Protopipe UI Composition Platform is designed to deliver dynamic, experiment-driven user interfaces composed of independently developed fragments (Render Function Artifacts, RFAs).

The platform must ensure:

- deterministic rendering across all users
- consistent experiment exposure
- controlled data flow between composition layers
- clear separation of concerns between composition and presentation

The architecture introduces multiple layers of indirection:

- experiment-based artifact routing
- template and slot composition
- remote fragment resolution (RFA execution)

Client-side rendering (CSR) introduces inherent challenges in such a system:

- non-deterministic rendering due to asynchronous client execution
- flickering and inconsistent experiment exposure
- delayed time-to-first-content
- increased complexity in coordinating experiments, state, and telemetry
- duplication of logic between client and server

A server-centric model provides stronger guarantees for:

- consistency
- observability
- control over execution

## Decision

The platform adopts a **Server-Side Rendering (SSR-first)** model as its primary rendering strategy.

This means:

- All pages are composed and rendered on the server before being delivered to the client
- The server is responsible for:
  - experiment assignment
  - template resolution
  - slot composition
  - execution of RFAs
- The client receives fully rendered HTML as the canonical output

Client-side logic is:

- optional
- non-authoritative
- limited to progressive enhancement

Client-side composition of fragments is explicitly not part of the core rendering model.

The canonical rendering flow is:

```text
Request
 → Experiment Assignment
 → Template Resolution
 → Slot Composition
 → RFA Execution (parallel)
 → HTML Response
 → Telemetry Emission
```
