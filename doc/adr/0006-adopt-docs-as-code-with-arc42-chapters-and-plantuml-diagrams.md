# 0006 – Adopt Docs-as-Code with arc42 Chapters and PlantUML Diagrams

Date: 2025-12-29

## Status

Accepted

## Context

Architecture documentation must remain accurate, reviewable, and closely aligned
with the implemented system.

External documentation systems or slide-based approaches tend to drift from
reality and are rarely kept up to date.

## Decision

Architecture documentation is maintained as **docs-as-code** within the
repository using:

- arc42 chapter structure
- Markdown files
- PlantUML diagrams rendered via CI

Documentation changes are reviewed and versioned together with code changes.

## Consequences

- Architecture documentation stays close to implementation.
- Diagrams are reproducible and version-controlled.
- Contributors can review documentation changes like code.
- Requires CI support and disciplined maintenance.

