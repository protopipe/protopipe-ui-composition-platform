# Project Instructions

## Architecture Documentation

Before making architectural or structurally relevant changes, read and follow
the canonical meta information and project architecture documentation:

- Canonical meta repository: `https://github.com/protopipe/noesis`
- ADRs: `doc/adr/README.md` and the relevant `doc/adr/*.md` records
- arc42: `doc/arc42/README.md` and the relevant `doc/arc42/*.md` chapters

Treat `protopipe/noesis` as an upstream source for canonical meta information
that may affect decisions, designs, terminology, and architectural direction.
If the repository is available locally, prefer reading that checkout directly.
If it is not available locally, use an authorized GitHub or MCP connection when
available. If neither is available and the task depends on that information, ask
for access or for the relevant excerpts before making the decision.

Treat accepted ADRs and documented arc42 content as architectural guidance for
this codebase. If code, configuration, tests, or documentation appear to
conflict with canonical meta information, an accepted ADR, or an arc42 chapter,
call out the mismatch before changing the architectural direction.

## Architecture Consistency Check

When a task touches architecture, deployment, runtime behavior, data flow,
rendering boundaries, experiments, interaction events, observability, or other
documented system concepts:

1. Read the relevant ADRs and arc42 chapters first.
2. Check whether `protopipe/noesis` contains canonical meta information relevant
   to the decision or design.
3. Check whether the documented decisions and architecture descriptions still
   match the implementation.
4. Keep code changes aligned with the canonical and documented architecture.
5. If the implementation has intentionally moved beyond the documentation,
   update the appropriate docs or propose a new ADR instead of silently drifting.

Use the repo-local Codex skill `skills/architecture-docs-governance/SKILL.md`
for architecture-document review and consistency work.
