# 0008 – Centralize Experiment Routing Before Rendering

Date: 2026-04-01

## Status

Accepted

## Context

The Protopipe UI Composition Platform treats experiments as a first-class
architectural concept that influences how user interfaces are delivered.

The system is composed of multiple layers:

- Pages (entry points)
- Templates (structure)
- Slots (composition points)
- Render Function Artifacts (RFAs) (rendering units)

Experiments must influence which artifacts are used without introducing
non-deterministic behavior or distributing logic across multiple layers.

If experiment decisions are made:

- inside RFAs
- inside templates
- or on the client

the system suffers from:

- inconsistent rendering behavior
- flickering and race conditions
- duplicated logic
- reduced observability
- loss of reproducibility

A single, authoritative point of decision is required to ensure consistent
behavior.

## Decision

Experiment routing is **centralized and executed before rendering begins**.

This means:

- Experiment assignment happens once per request
- The result of the assignment determines all subsequent artifact selection

Routing is applied at the composition level:

- Page
- Template
- Slot
- RFA

The system resolves all experiment decisions before any rendering logic is executed.

RFAs:

- MUST NOT perform experiment assignment
- MUST NOT contain branching logic based on experiment variants
- MUST remain deterministic given their inputs

Templates and slots:

- define structure only
- MUST NOT include experiment decision logic

The canonical flow is:

```text
Request
 → Experiment Assignment (centralized)
 → Artifact Selection (Page / Template / Slot / RFA)
 → Composition & Rendering
```

## Consequences

### Positive

- Deterministic and reproducible rendering behavior
- No flickering or inconsistent experiment exposure
- Single source of truth for experiment decisions
- Clear separation between decision logic and rendering logic
- Improved observability and debugging capabilities

### Negative / Risks

- Increased responsibility on the composition layer
- Requires strict enforcement of architectural boundaries
- Potential misuse if developers bypass central routing

### Mitigations

- Enforce constraints through code reviews and tooling
- Provide clear architectural guidelines and examples
- Validate that RFAs remain free of experiment logic
- Centralize experiment assignment in a dedicated component
