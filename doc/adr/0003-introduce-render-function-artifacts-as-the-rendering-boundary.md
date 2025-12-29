# 0003 – Introduce Render Function Artifacts as the Rendering Boundary

Date: 2025-12-29

## Status

Accepted

## Context

Frontend teams must be able to work independently, using different technologies
and release cycles, without introducing shared frontend baselines or tight
coupling.

Traditional frontend integration approaches often rely on shared runtimes,
framework versions, or client-side composition, which does not scale well
organizationally.

A stable, testable rendering boundary is required.

## Decision

We introduce **Render Function Artifacts (RFAs)** as the primary rendering
boundary.

An RFA exposes a pure rendering contract:

```
render(data, context)->html(+meta)
```

RFAs are:
- Technology-agnostic internally
- Independently buildable and testable
- Executed server-side by the frontend composer

## Consequences

- Teams can choose internal frontend technologies freely.
- Rendering becomes deterministic and testable.
- Shared frontend baselines are avoided.
- Server-side execution requires isolation and resource control.

