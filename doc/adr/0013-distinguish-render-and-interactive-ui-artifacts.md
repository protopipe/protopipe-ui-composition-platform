# 0013 – Distinguish Render and Interactive UI Artifacts

Date: 2026-04-01

## Status

Accepted

## Context

The Protopipe UI Composition Platform composes user interfaces from
independently developed artifacts.

A single abstraction for all UI components leads to either:

- overly restrictive contracts (limiting interactivity), or
- overly complex contracts (reducing testability and clarity)

The platform must support:

- deterministic server-side rendering (SSR)
- interactive client-side behavior
- explicit and testable interaction contracts
- low coupling between components

A clear separation of responsibilities is required.

## Decision

The platform defines two types of UI artifacts:

### 1. Render Function Artifacts (RFAs)

RFAs are responsible for deterministic rendering.

They:

- take explicit input and context
- produce HTML (and optional metadata)
- are executed during server-side rendering

RFAs:

- MUST be deterministic
- MUST NOT handle interaction logic
- MUST NOT depend on runtime event streams

---

### 2. Interactive Function Artifacts (IFAs)

IFAs extend RFAs with interaction capabilities.

They:

- are initialized with a deterministic initial state
- react to events via explicit event contracts
- produce updated UI representations and interaction events

An IFA follows the model:

```text
initialize(initialState, context)
  → { render(), handle(event) }
```

Where:

- `render()` produces the current UI representation
- `handle(event)` processes an event and returns:
  - updated UI state (via render)
  - emitted interaction events

---

### Interaction Model

IFAs:

- MAY consume:
  - interaction events
  - business events

- MUST only emit:
  - interaction events

IFAs:

- MUST NOT emit business events
- MUST NOT perform business validation
- MUST NOT own authoritative business state

---

### Architectural Constraints

- All dependencies between UI artifacts MUST be expressed via events
- Direct component-to-component communication is not allowed
- State must not be implicitly shared between artifacts
- Interaction behavior must be deterministic based on:
  - initial state
  - sequence of events

## Consequences

### Positive

- Clear separation between rendering and interaction
- Strong testability of both RFAs and IFAs
- Reduced coupling between UI components
- Explicit and observable interaction flows
- Compatibility with SSR and offline-first architectures

### Negative / Risks

- Requires discipline in defining event contracts
- Increased conceptual overhead (RFA vs IFA distinction)
- Potential duplication between render and interaction logic

### Mitigations

- Provide clear guidelines and examples for both artifact types
- Keep RFAs as the default for simple components
- Use IFAs only when interaction complexity justifies it
- Provide tooling for event inspection and testing
