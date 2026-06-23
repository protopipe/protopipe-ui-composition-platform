# 05 – Building Block View

## Core Building Blocks

### Page
The smallest deployable frontend unit, bound to a route or URL.

A Page declares whether it renders a deterministic Render Function Artifact
(RFA) or an Interactive Function Artifact (IFA). RFA Pages must not define
interaction configuration. IFA Pages require explicit interaction configuration
for client message delivery and event reconciliation.

A Page may also declare a Proxy Page delivery mode (see
[ADR-0020](../adr/0020-support-proxy-page-markers-as-stable-and-experimental-composition-points.md)).
In that mode the Page owns the route, upstream origin, and stable accepted
marker replacements for a monolith-backed page.

### Template
An immutable layout definition for a Page, containing named Slots.

### Slot
A named insertion point within a Template, governed by a contract and policies.

### Proxy Marker
An HTML comment based composition point in an upstream monolith response.

Proxy Markers are inert when no replacement is active. Accepted marker
replacements are part of the Page definition. Candidate marker replacements may
be introduced by Experiment variants and promoted into Page configuration after
validation.

### Fragment
A deployable unit that fills a Slot with rendered content.

### Render Function Artifact (RFA)
A packaged rendering unit exposing a pure function:

render(data, context) -> html (+meta)

### Interactive Function Artifact (IFA)
An interaction-capable UI artifact that extends deterministic rendering with
explicit event handling.

An IFA is initialized from server-rendered state and context, consumes
interaction and business events, and may emit interaction events. It does not
own authoritative business state and must not emit business events.

### Experiment
A governance resource defining assignment rules and artifact overrides.

Experiments may introduce candidate Proxy Marker replacements for A/B tests,
canaries, and preview validation. These replacements are applied to the
effective Page configuration after assignment and before rendering.

## Responsibilities

- Teams own RFAs and contracts.
- The platform owns composition, routing, caching, and experiments.
- Management owns experiment intent and KPI evaluation.
