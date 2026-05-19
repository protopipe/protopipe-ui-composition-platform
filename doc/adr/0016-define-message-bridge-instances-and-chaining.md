# 0016 – Define Message Bridge Instances and Chaining

Date: 2026-05-19

## Status

Accepted

## Context

ADR-0011, ADR-0012, ADR-0014, and ADR-0015 define an event-based interaction
model, explicit interaction lifecycle states, buffered delivery, and the
separation between experiment assignment and interaction channel selection.

The platform now needs a common protocol model that works across multiple
runtime locations:

- browser and PWA clients,
- server-side Message Bridge instances,
- edge or local-network Message Bridge instances,
- chained Message Bridge deployments for hybrid cloud, mobile, offline, or
  constrained connectivity scenarios.

The frontend message library should therefore not be treated as a one-off
client helper. It is a client-side bridge instance with a smaller capability
profile.

## Decision

Message Bridge is a protocol and runtime pattern, not only a single backend
service.

The platform distinguishes the following bridge profiles:

- **Client-side Message Bridge**: runs in the browser or PWA, observes
  interaction events, buffers locally when needed, propagates authentication
  context, sends event batches upstream, receives response events, and
  reconciles UI state.
- **Server-side Message Bridge**: acts as policy gate, durable delivery
  endpoint, channel resolver, response-event endpoint, and internal integration
  adapter.
- **Edge Message Bridge**: may run close to devices, branches, field workers,
  or constrained networks, buffer events locally, and synchronize with an
  upstream bridge.

Message Bridges MAY be chained. A downstream bridge may receive batches from an
upstream bridge, persist them durably, apply local delivery policies, and
forward them later.

Each Message Bridge layer owns its local delivery identity. Message identifiers,
cursors, offsets, partitions, batch identifiers, and replay positions are scoped
to the bridge layer or cluster that issues them. A chained bridge may preserve
upstream identifiers as metadata, but it MUST assign and manage its own local
delivery identifiers for its consumers.

A consumer is registered against exactly one Message Bridge layer or cluster for
a given stream subscription. It MUST NOT assume that cursors or partitions from
one bridge layer can be used directly against another bridge layer. Moving a
consumer between bridge layers requires explicit migration or resynchronization
that maps the old position to the new layer's local delivery position.

Every Message Bridge implementation MUST share a common core feature set:

- stable message identifiers,
- correlation identifiers,
- message type classification,
- optional ordering key and partition metadata,
- durable buffering for accepted messages according to its capability profile,
- at-least-once delivery,
- idempotent batch acceptance,
- acknowledgement of delivered batches,
- retry after temporary delivery failures,
- clear separation of interaction events, response events, business events, and
  technical events.

Not every implementation must support every advanced feature. In particular:

- Client-side bridges may use browser storage such as IndexedDB instead of a
  broker.
- Edge bridges may support only a subset of business-event replay.
- Server bridges provide the full policy and integration surface.
- Business-event sourcing is optional and only enabled for configured streams.

## Consequences

### Positive

- Offline-capable PWAs and server-side delivery use the same conceptual model.
- Hybrid cloud, edge, mobile, and branch-office topologies can be modeled as
  bridge chains.
- The frontend message library gets a durable architectural contract instead of
  being an implementation detail.
- Bridge capabilities can evolve independently by profile.

### Negative / Risks

- The term Message Bridge becomes broader and needs precise capability
  descriptions.
- Chaining adds operational complexity around retries, idempotency, duplicate
  delivery, and observability.
- Consumers cannot trivially move between bridge layers because delivery
  positions are layer-local.
- Client-side buffering must be treated as cache or pending intent, not as
  authoritative business state.

### Mitigations

- Define explicit capability profiles for client, server, and edge bridges.
- Require idempotent message identifiers and batch identifiers.
- Preserve upstream identifiers as metadata when bridging, while exposing local
  cursors and partitions to local consumers.
- Provide explicit migration or resynchronization procedures before moving a
  consumer to another bridge layer.
- Keep domain ownership outside the bridge. Domain services remain the source of
  truth.
- Make hop metadata, delivery lag, retries, and dead-letter states observable.
