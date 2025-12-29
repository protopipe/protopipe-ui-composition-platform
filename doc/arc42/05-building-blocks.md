# 05 – Building Block View

## Core Building Blocks

### Page
The smallest deployable frontend unit, bound to a route or URL.

### Template
An immutable layout definition for a Page, containing named Slots.

### Slot
A named insertion point within a Template, governed by a contract and policies.

### Fragment
A deployable unit that fills a Slot with rendered content.

### Render Function Artifact (RFA)
A packaged rendering unit exposing a pure function:

render(data, context) -> html (+meta)

### Experiment
A governance resource defining assignment rules and artifact overrides.

## Responsibilities

- Teams own RFAs and contracts.
- The platform owns composition, routing, caching, and experiments.
- Management owns experiment intent and KPI evaluation.

