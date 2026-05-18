---
name: architecture-docs-governance
description: Use when working in this repository on architectural, structural, deployment, runtime, rendering, experiment, interaction-event, observability, design, decision, or documentation changes that must be checked against protopipe/noesis, ADRs, and arc42 documentation.
---

# Architecture Docs Governance

Use this skill for changes or reviews that may affect the documented
architecture of this repository.

## Required Reading

Start with the canonical local architecture indexes:

- `doc/adr/README.md`
- `doc/arc42/README.md`

Then read only the ADRs and arc42 chapters relevant to the task. Prefer focused
reading over loading all documentation by default.

Also check the canonical meta repository when a task depends on product,
terminology, design, decision, or platform-level context:

- `https://github.com/protopipe/noesis`

If `protopipe/noesis` is checked out locally, read that checkout. If it is not
local, use an authorized GitHub or MCP connection when available. If neither is
available and the task depends on that context, ask for access or relevant
excerpts before deciding.

## Workflow

1. Identify the architectural surface of the task: rendering boundaries,
   experiments, event delivery, Kubernetes resources, deployment, runtime flow,
   observability, consent behavior, docs-as-code, design language, or canonical
   meta-models.
2. Read the matching ADRs in `doc/adr/` and matching arc42 chapters in
   `doc/arc42/`.
3. Check `protopipe/noesis` for relevant canonical meta information when the
   task touches decisions, designs, terminology, platform concepts, or product
   direction.
4. Inspect the relevant implementation, configuration, tests, and docs.
5. Compare implementation against canonical meta information, documented
   decisions, and architecture descriptions.
6. If documentation and implementation agree, keep the change aligned with that
   guidance.
7. If they disagree, make the mismatch explicit:
   - update arc42 when the documented description is stale,
   - add or propose a new ADR when an accepted architectural decision needs to
     change,
   - update or reference `protopipe/noesis` when canonical meta information is
     missing or supersedes local project assumptions,
   - or adjust the implementation when it accidentally drifted from the
     accepted architecture.

## Documentation Rules

- Do not retroactively rewrite accepted ADRs to change their meaning.
- Use a new ADR to supersede or amend an accepted decision.
- Keep arc42 chapters descriptive and current with the implemented system.
- Do not duplicate large parts of `protopipe/noesis`; reference the canonical
  source and mirror only the local implications needed by this repository.
- When changing code and docs together, mention which ADRs or arc42 chapters were
  checked and whether `protopipe/noesis` was consulted.

## Useful Starting Points

- Architecture decisions: `doc/arc42/09-architecture-decisions.md`
- Building blocks: `doc/arc42/05-building-blocks.md`
- Runtime behavior: `doc/arc42/06-runtime-view.md`
- Deployment: `doc/arc42/07-deployment-view.md`
- Cross-cutting concepts: `doc/arc42/08-cross-cutting-concepts.md`
- Risks and technical debt: `doc/arc42/11-risks-and-technical-debt.md`
