# 0012 – Define Interaction Event Lifecycle

Date: 2026-04-01

## Status

Accepted

## Context

The platform supports asynchronous interaction patterns, including:

- server-side rendering
- event-based UI interaction (see ADR-0012)
- offline-capable clients (e.g. PWAs)
- deferred delivery of interaction events

To ensure predictable system behavior, a consistent lifecycle for interaction
events is required.

Without a defined lifecycle, the system risks:

- unclear event states
- inconsistent UI behavior
- difficult debugging and observability
- unreliable offline synchronization

## Decision

Interaction events follow a defined lifecycle:

```text
Observed → Buffered → Delivered → Processed → Resolved
```

### Observed

- Event is generated in the UI
- Represents a user interaction
- Not yet sent to backend

### Buffered (optional)

- Event is stored locally (e.g. offline, batching)
- Not yet delivered

### Delivered

- Event is sent to backend
- Transport acknowledged
- Not yet processed by business logic

### Processed

- Backend validates and processes the event
- Results in:
  - business event
  - or technical rejection

### Resolved

- UI receives and processes the result
- Optimistic state is confirmed or corrected

## Additional Rules

- Interaction events are **not replayed**
  - they are collected and delivered later if necessary

- Delivered does not imply acceptance
  - UI must wait for resolved state

- Business events are the only source of authoritative state

- Technical failures (e.g. invalid schema, auth failure) are not business events

## Consequences

### Positive

- Clear and observable event flow
- Support for offline and deferred interaction models
- Improved debugging and tracing capabilities
- Predictable reconciliation of optimistic UI

### Negative / Risks

- Requires event identifiers and correlation handling
- Additional complexity in client-side state management

### Mitigations

- Use unique event identifiers
- Use correlation IDs to match responses
- Provide developer tooling for event lifecycle inspection
