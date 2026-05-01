# 0014 – Use Buffered Polling-Based Event Delivery

Date: 2026-04-01

## Status

Accepted

## Context

The Protopipe UI Composition Platform uses event-based interaction between
UI artifacts and backend services.

The platform must support:

- deterministic UI rendering (SSR)
- event-driven interaction (see ADR-0011)
- offline-capable clients (e.g. PWAs)
- scalable and stateless infrastructure
- low operational complexity

Push-based communication mechanisms (e.g. WebSockets) introduce:

- long-lived connections
- scaling and load-balancing complexity
- difficult failure recovery
- increased infrastructure coupling

At the same time, interaction events may be temporarily unavailable for delivery
due to:

- offline clients
- network interruptions
- backend delays

A clear delivery model and responsibility model is required.

## Decision

The platform uses **buffered, polling-based event delivery** between frontend
and backend components.

Events flow through the following stations:

```text
UI Artifact
 → Client-side Message Bridge
 → Server-side Message Bridge (Composition Service)
 → Business Service
 → Server-side Message Bridge
 → Client-side Message Bridge
 → UI Artifact
```

Each station may buffer events temporarily.

Clients retrieve pending events via polling:

```text
Client polls /messages
 → server returns pending events
 → client-side message bridge distributes events
```

Push sockets are not required for frontend-backend communication.

---

## Event Lifecycle

Interaction events follow this lifecycle:

```text
Observed → Buffered → Delivered → Processed → Resolved
```

### Observed

- Event is created by a UI artifact
- Represents a user interaction

### Buffered

- Event is temporarily stored
- May occur at any buffer station

### Delivered

- Event is transmitted to the next station
- Delivery does not imply acceptance

### Processed

- Event is processed by the responsible component
- Results in:
  - business event
  - or technical event

### Resolved

- UI receives sufficient information to reconcile its state

---

## Event Types

```text
Interaction Event:
  emitted by UI artifacts
  represents user interaction or UI intent

Business Event:
  emitted by business services
  represents validated business state

Technical Event:
  emitted by infrastructure
  represents transport, validation, or failure outcomes
```

---

## Offline and Deferred Delivery

Interaction events are not replayed.

They may be:

- observed
- collected
- buffered
- delivered later

These events represent interactions not yet processed by backend services.

---

## Responsibilities

### UI Artifact

Responsible for:

- emitting interaction events
- consuming interaction and business events
- reconciling optimistic UI state

Not responsible for:

- business validation
- authoritative state
- semantic exactly-once guarantees

---

### Client-side Message Bridge

Responsible for:

- collecting interaction events
- buffering events during disconnection
- delivering events to server-side bridge
- distributing incoming events to UI artifacts
- preserving event and correlation identifiers

Not responsible for:

- business logic
- business validation
- interpreting event meaning

---

### Server-side Message Bridge (Composition Service)

Responsible for:

- receiving interaction events from clients
- routing events to business services
- buffering events temporarily
- returning pending events to clients via polling
- performing technical deduplication where possible
- emitting technical events on failure

Not responsible for:

- business validation
- business decisions
- authoritative state
- semantic exactly-once guarantees

---

### Business Service

Responsible for:

- validating interaction events
- owning authoritative business state
- deciding outcomes
- emitting business events
- ensuring semantic idempotency and exactly-once behavior where required

Business services are the only components allowed to emit business events.

---

### Platform / Operations

Responsible for:

- defining retention limits for buffered events
- defining retry and failure behavior
- making event flow observable
- preventing unbounded queues
- ensuring failures do not cascade

---

## Guarantees and Constraints

- Message bridges MAY provide technical deduplication
- Message bridges MUST NOT guarantee semantic exactly-once processing
- Semantic exactly-once is a responsibility of business services
- Each event MUST carry identifiers for correlation and tracing
- Buffered events are not guaranteed to persist indefinitely
- Event loss or failure MUST be observable

---

## Consequences

### Positive

- Supports offline-capable clients and PWAs
- Avoids complexity of push sockets
- Enables stateless scaling of infrastructure
- Clear separation of responsibilities
- Strong testability through event flows
- Predictable failure handling

### Negative / Risks

- Increased latency compared to push models
- Requires explicit handling of buffering and retries
- Potential for duplicate or delayed events
- Requires UI reconciliation logic

### Mitigations

- Use event IDs for deduplication
- Use correlation IDs for tracing
- Define retention and retry policies
- Prefer eventual consistency over synchronous coupling
- Provide tooling for event inspection and debugging
