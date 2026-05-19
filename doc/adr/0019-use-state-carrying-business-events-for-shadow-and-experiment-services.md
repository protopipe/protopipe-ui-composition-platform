# 0019 – Use State-Carrying Business Events for Shadow and Experiment Services

Date: 2026-05-19

## Status

Accepted

## Context

ADR-0008 keeps experiment assignment centralized in the Composer. ADR-0015 keeps
the Message Bridge responsible for interaction channel selection without making
it an experiment assignment engine. ADR-0018 defines governed business-event
delivery with typed schemas, field-level classification, redaction, and optional
business-event sourcing.

The platform must support services that run in shadow mode or serve only a
subset of users during an experiment. Such services may later replace the active
service or run in parallel with it while receiving the same business-object
state changes.

If those services need to follow up every event with a domain-service read, the
experiment and integration model becomes fragile:

- shadow services observe a different state than the active service when reads
  happen at different times,
- experiment variants leak into service implementation through ad-hoc read
  logic,
- services depend on synchronous availability of the active domain service,
- partial rollout becomes harder to reason about,
- eventual consistency becomes non-deterministic rather than event-carried.

## Decision

Business events are the synchronization contract for services that run in shadow
mode, replacement mode, or experiment mode.

Business events MUST carry the business-object state required by their intended
consumers to update their local view or process the transition. Consumers MUST
NOT depend on a follow-up read from the active domain service to interpret the
event.

The Message Bridge helps prevent experiment complexity from leaking into
services:

- it delivers the same ordered business-event sequence to active, shadow, and
  experiment consumers according to their registration,
- it applies consumer-specific field-level redaction without changing cursor
  continuity,
- it keeps experiment routing and channel selection outside the business service
  implementation,
- it supports parallel consumers so that a shadow service can observe the same
  business-object stream before it becomes active,
- it can replay sourced business-event streams to resynchronize a consumer when
  sourcing and retention are enabled.

A service may be only a partial source of truth for a business-object domain.
For example, a shadow or experiment service may own a local projection, derived
state, or candidate implementation while another service remains authoritative
for writes. In that case, the service MUST listen to the business events of its
business objects and keep its local state eventually consistent through the
event stream.

Switching traffic from the active service to a shadow or experiment service is
only safe after the candidate service has consumed the required business-event
history and reached the expected cursor, partition, or version position for the
business objects it will serve.

## Consequences

### Positive

- Shadow services can be warmed up and validated before receiving user traffic.
- Experiment variants can run in parallel without embedding experiment branching
  in service code.
- Service replacement can be based on event-carried state and observable cursor
  progress.
- Business-object synchronization remains deterministic under eventual
  consistency.
- The active service does not become a synchronous read dependency for every
  event consumer.

### Negative / Risks

- Business events must be designed as useful state-carrying contracts, not as
  empty notifications.
- Event schemas may become larger because they carry the state required by
  intended consumers.
- Producers must understand which consumers are intended and which fields are
  required by the contract.
- Consumers must implement idempotency and version handling.

### Mitigations

- Define intended consumers and required fields as part of each business-event
  schema.
- Require field classification for every event property.
- Require version or sequence information for state-changing business events.
- Expose consumer lag, cursor position, partition position, and redaction
  decisions.
- Treat activation of a shadow or experiment service as a controlled rollout
  step that checks event-consumption progress.
