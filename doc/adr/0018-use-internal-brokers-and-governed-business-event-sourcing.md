# 0018 – Use Internal Brokers and Governed Business Event Sourcing

Date: 2026-05-19

## Status

Accepted

## Context

ADR-0015 selected the Message Bridge as channel selection and durable delivery
component while keeping experiment assignment in the Composer. The initial
vertical slice uses RabbitMQ to buffer and deliver interaction events.

For business events, the platform needs stronger governance:

- event payloads may contain personal, confidential, or secret data,
- consumers have different trust levels and scopes,
- replay must preserve cursor and ordering semantics,
- event details must not be spread across many consumer-specific queues or logs,
- broker topology must not become a second, decentralized authorization model.

This aligns with the canonical Noesis mechanics:

- `M-006_eventual-consistency_over_global-process-engines`,
- `M-013_event-carried-versioning_over_best-effort-consistency`,
- `M-014_classified-domain-objects_over_implicit-sensitivity`.

The platform also needs an implementation path that remains easy for ordinary
service teams and does not force every service to implement event sourcing.

## Decision

Broker technologies are internal Message Bridge adapters, not the public
consumer contract.

RabbitMQ is used internally for Message Bridge buffering, delivery work queues,
retry, dead-lettering, and outbox/inbox processing.

Kafka or a compatible event-log technology MAY be used internally for
business-event sourcing. Business-event sourcing is optional and is enabled only
for configured business-event streams. If sourcing is not enabled, committed
messages may be forgotten by the bridge according to retention policy.

Business events use typed schemas. Every message property MUST declare a
classification that describes its sensitivity and disclosure rules. The schema
format may be Avro, JSON Schema, or another schema language, but the schema must
support Protopipe field metadata.

Business events MUST carry the state needed by their intended consumers so that
consumers do not need a follow-up read from the domain service to interpret the
event. This state is still governed by field-level classification and redaction.
Consumers that are not allowed to see a field receive a redacted payload for the
same cursor instead of being forced into an inconsistent readback pattern.

State-changing business events MUST carry version information sufficient to
detect out-of-order or conflicting state transitions. Event sourcing does not
become mandatory for every service, but version discipline is mandatory where a
business event represents state evolution.

Consumers must be registered as trusted consumers with explicit scopes,
capability profile, delivery endpoint, batch limits, and replay permissions.

During delivery, the Message Bridge performs field-level redaction according to
consumer permissions:

- events are not filtered out only because a consumer lacks access to their
  payload fields,
- cursor, partition, event id, and required envelope metadata remain visible
  when the consumer is allowed to observe the stream,
- fields the consumer may not read are omitted,
- if no payload fields are visible, the event is delivered with an empty payload
  object.

This preserves cursor continuity and ordering while enforcing data minimization.

The Message Bridge guarantees at-least-once delivery only. Consumers are
responsible for semantic idempotency and semantic exactly-once behavior.

## Consequences

### Positive

- Broker choice remains replaceable behind the Message Bridge protocol.
- Business-event governance is centralized and auditable.
- Replay can be enabled where it is valuable without forcing all services into
  event sourcing.
- Sensitive data is not duplicated into consumer-specific logs or queues.
- Cursor-preserving redaction keeps replay and ordering understandable.
- Consumers do not depend on domain-service readback to interpret business
  events.

### Negative / Risks

- The Message Bridge becomes a critical policy enforcement component.
- Field-level classification and redaction add schema and runtime complexity.
- Consumers need clear tooling to understand why fields are omitted.
- Kafka/event-log operation adds cost when sourcing is enabled.
- Business-event schemas can become larger because events carry the state needed
  by their intended consumers.
- Producers must understand the disclosure impact of fields they include in
  business events.

### Mitigations

- Keep business events purposeful: include the state required for the event
  contract, classify every field, and rely on redaction instead of follow-up
  reads for sensitive details.
- Start with a small classification vocabulary and extend it carefully.
- Validate schemas before activation.
- Redact logs and traces based on the same schema metadata.
- Make the effective schema, consumer scopes, policy version, and redaction
  decisions inspectable.
