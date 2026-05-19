# 0017 – Use Buffered HTTP Batch Delivery for Message Bridges

Date: 2026-05-19

## Status

Accepted

## Context

ADR-0014 introduced buffered polling-based delivery for frontend interaction
flows. ADR-0016 generalizes the Message Bridge pattern across client-side,
server-side, and edge bridge instances.

For business services and bridge-to-bridge delivery, a default protocol is
needed that is easy for teams to implement, works across hybrid cloud and
enterprise network boundaries, and does not require every team to understand
broker-specific concepts such as Kafka consumer groups, RabbitMQ bindings, or
broker offsets.

The protocol must support offline and temporarily unreachable consumers while
keeping delivery semantics clear.

## Decision

The default Message Bridge delivery protocol is buffered HTTP batch delivery.

A Message Bridge delivers messages to registered downstream consumers or bridge
instances via HTTP webhook-style batch requests. Each batch includes:

- stream or channel identifier,
- batch identifier,
- from-cursor and to-cursor,
- optional partition identifier,
- message list,
- schema and policy metadata where relevant.

Consumers acknowledge a batch by returning a successful HTTP response. A batch
is committed for that consumer only after acknowledgement.

If a batch is not acknowledged, the Message Bridge retries delivery according to
its retry policy. Consumers MUST treat delivery as at-least-once and implement
semantic idempotency.

The protocol also supports cursor-based replay. A registered consumer or
downstream bridge may request that the Message Bridge resend messages from one
cursor position to another cursor position. Replay is only available for streams
whose retention or sourcing capability profile supports it.

Payload streaming MAY use newline-delimited JSON (NDJSON/JSON Lines) for
incremental processing. Each line is a complete event representation. If a
connection breaks after complete lines, the consumer can continue from the last
acknowledged cursor.

Batch size MUST be configurable per consumer. Configuration MAY include:

- maximum number of messages,
- maximum bytes per batch,
- maximum delivery interval,
- retry backoff,
- dead-letter policy,
- replay capability,
- accepted content types.

## Consequences

### Positive

- Consumers can be normal HTTP services.
- Delivery works well with enterprise networking, hybrid cloud, and bridge
  chaining.
- Broker-specific APIs remain internal implementation details.
- The Message Bridge can centrally apply policy, redaction, auditing, retry, and
  backpressure.
- Consumers can recover missed batches through cursor replay when supported.

### Negative / Risks

- Consumers must expose reachable HTTP endpoints or use a downstream bridge that
  does.
- The Message Bridge becomes responsible for retry scheduling and delivery
  state.
- At-least-once delivery means consumers must handle duplicates.
- High-volume streams may need more specialized pull or streaming delivery
  later.

### Mitigations

- Provide a small consumer SDK for webhook validation, idempotency, and
  acknowledgement handling.
- Support bridge chaining for consumers that are not directly reachable.
- Keep pull/streaming APIs as optional advanced capabilities.
- Make delivery lag, retry counts, and dead-lettered batches visible.
