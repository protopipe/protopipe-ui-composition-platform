# 0023 – Use Version-Fenced POST Results for Eventual Consistency

Date: 2026-05-24

## Status

Accepted

## Context

ADR-0022 defines the basic Composer POST flow as `POST` followed by `303 See
Other` and a redirected GET result page. That flow solves browser reload and
rendering-boundary concerns, but it does not by itself guarantee read-your-write
semantics for the redirected result page.

ADR-0018 defines governed business-event delivery and requires state-changing
business events to carry version information. ADR-0019 defines state-carrying
business events as the synchronization contract for shadow and experiment
services. It also states that switching traffic to shadow or experiment services
is safe only after they have reached the expected cursor, partition, or version
position.

This matters for POST results. A POST may be processed by a service selected by
the effective page configuration or an experiment override. The redirected GET
result page may then read from another service, such as a default read model,
shadow service, or experiment service. If that read service has not yet consumed
the business event produced by the POST, the result page could render stale or
incorrect state.

The Composer must therefore have a way to connect the accepted user intent to
the version of business state that result rendering must observe, without
storing submitted data or sensitive result state itself.

## Decision

POST services that require action-specific result rendering MUST return a write
acknowledgement that can be referenced against business-event progress.

The acknowledgement MUST contain non-sensitive metadata sufficient to describe
the accepted write position. At minimum, this includes:

- stream or business-object type,
- business key or subject,
- ordering key or partition key,
- version, cursor, or equivalent event position.

The contract SHOULD use `partitionKey` or `orderingKey` for the stable
business-level ordering dimension. Broker-specific partition numbers are
adapter details and MUST NOT become the public Composer contract.

An acknowledgement may look like:

```json
{
  "stream": "contact-requests",
  "subject": "contactRequest",
  "businessKey": "req-123",
  "partitionKey": "contact:req-123",
  "version": 7,
  "eventId": "evt-789"
}
```

The Composer may include non-sensitive acknowledgement fields in the 303
redirect target when the target page is configured to accept them. Redirect URLs
MUST NOT contain submitted form data, personal data, secrets, or sensitive
result payloads.

The redirected GET page may use the acknowledgement as a version fence. A
version-fenced data dependency asks a read service for state at least as new as
the acknowledged business version or cursor. The Composer MAY wait for a
version-fenced data dependency only within an explicit bounded timeout. If the
required version cannot be observed within the timeout, the page MUST render an
explicit pending or degraded result according to its page configuration.

The Composer MUST NOT store submit result state to bridge POST and redirected
GET requests. Backend services own business state, projections, and event
progress.

## Consequences

### Positive

- Redirected result pages can render the correct version of business state even
  under eventual consistency.
- Experiment and shadow services can process or read POST results without
  embedding experiment branching in UI artifacts.
- Side effects remain governed by configured backend services and
  business-event streams.
- The Composer remains stateless with respect to submitted data and result
  payloads.
- The design aligns POST result rendering with ADR-0018 and ADR-0019 version and
  partition discipline.

### Negative / Risks

- POST services need an explicit write acknowledgement contract.
- Read services that participate in action-specific result rendering need to
  support version-fenced reads or equivalent readiness semantics.
- Result rendering can become slower when the Composer waits for projections to
  catch up.
- Redirect URLs may still leak non-sensitive identifiers through logs and
  browser history if configured carelessly.

### Mitigations

- Keep redirect parameters opaque, non-sensitive, and minimal.
- Prefer generic confirmation pages when action-specific state is not needed.
- Require bounded wait budgets for all version-fenced reads.
- Render explicit pending/degraded result states when the required version is
  not yet visible.
- Expose lag, cursor, partition key, version, and timeout metrics for
  operational diagnosis.
