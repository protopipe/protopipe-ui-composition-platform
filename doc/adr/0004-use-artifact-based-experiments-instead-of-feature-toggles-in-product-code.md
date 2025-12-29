# 0004 – Use Artifact-based Experiments Instead of Feature Toggles in Product Code

Date: 2025-12-29

## Status

Accepted

## Context

Feature toggles embedded in product code often lead to long-lived conditional
logic, increased complexity, and incomplete cleanup after experiments end.

The platform is designed around experiments as first-class delivery mechanisms
and supports routing at the artifact level.

## Decision

Experiments are implemented by **routing to different artifact implementations**
rather than toggling behavior inside product code.

Snapshot artifacts built from feature branches can be referenced directly by
experiments and promoted or discarded without modifying merged code.

## Consequences

- Product code remains clean and toggle-free.
- Definition of Done is reached at merge time.
- Experiments become reproducible and auditable.
- Artifact lifecycle management (retention, cleanup) becomes necessary.

