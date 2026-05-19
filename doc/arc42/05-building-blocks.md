# 05 – Building Block View

## Core Building Blocks

### Page
The smallest deployable frontend unit, bound to a route or URL.

A Page declares whether it renders a deterministic Render Function Artifact
(RFA) or an Interactive Function Artifact (IFA). RFA Pages must not define
interaction configuration. IFA Pages require explicit interaction configuration
for client message delivery and event reconciliation.

### Template
An immutable layout definition for a Page, containing named Slots.

### Slot
A named insertion point within a Template, governed by a contract and policies.

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

## Responsibilities

- Teams own RFAs and contracts.
- The platform owns composition, routing, caching, and experiments.
- Management owns experiment intent and KPI evaluation.
