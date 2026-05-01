# 0011 – Use Event-Based Interaction Model for UI Artifacts

Date: 2026-04-01

## Status

Accepted

## Context

The Protopipe UI Composition Platform aims to minimize coupling between
UI components while enabling independent development, testability, and
scalable composition.

Traditional frontend architectures often rely on:

- direct component-to-component communication
- shared mutable state (global stores)
- implicit dependencies between UI elements

These approaches lead to:

- tight coupling
- reduced testability
- hidden dependencies
- increased cognitive complexity

A consistent and explicit interaction model is required.

## Decision

UI artifacts interact exclusively through **explicit event contracts**.

This means:

- UI artifacts MAY:
  - emit interaction events
  - consume interaction events
  - consume business events

- UI artifacts MUST NOT:
  - directly call or depend on other UI artifacts
  - share implicit global state
  - emit business events

Interaction events:

- describe observed user interactions
- represent UI-local intent
- do not imply accepted business state

Business events:

- are emitted exclusively by backend services
- represent validated and accepted business state changes

The platform runtime (e.g. composer, event bridge):

- routes events between artifacts
- remains stateless with respect to business logic
- does not interpret or transform event semantics

## Consequences

### Positive

- Strong decoupling between UI artifacts
- Clear and explicit interaction boundaries
- Improved testability through event-driven contracts
- Easier reasoning about system behavior
- Compatibility with offline and asynchronous execution models

### Negative / Risks

- Increased need for clear event naming and versioning
- Potential event proliferation
- Requires discipline in defining event semantics

### Mitigations

- Establish event naming conventions
- Version events explicitly when needed
- Provide tooling for event inspection and debugging
