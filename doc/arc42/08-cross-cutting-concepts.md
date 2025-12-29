# 08 – Cross-cutting Concepts

## Render Function Artifact ABI

- Pure rendering contract
- No side effects except telemetry
- Deterministic output
- Technology-agnostic internal implementation

## Experimentation

- Experiments route artifact implementations.
- No feature toggles inside components.
- Snapshot artifacts are pinned for reproducibility.

## Testability and Shift-left

- CDCTs validate frontend-backend contracts.
- Storybook serves as integration and documentation hub.
- RFAs are executable locally with verified mock data.

## Security and Isolation

- RFAs are isolated at execution level.
- Limited context exposure.
- No shared global state.

