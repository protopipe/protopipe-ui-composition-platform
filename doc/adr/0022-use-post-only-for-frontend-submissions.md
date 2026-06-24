# 0021 – Use POST Only for Frontend Submissions

Date: 2026-05-24

## Status

Accepted

## Context

ADR-0007 defines an SSR-first rendering model. ADR-0008 keeps experiment
assignment centralized in the Composer. ADR-0011 and ADR-0014 define UI
interaction flows as user intent delivery rather than direct domain-state
mutation by UI artifacts.

RFAs and IFAs must not own business-side effects. Still, browser forms and UI
artifacts need a simple way to submit user intents to the Composer so that the
Composer can apply page routing, experiment decisions, service policy, and
observability before a backend service processes the intent.

Allowing frontend artifacts to use resource-oriented HTTP verbs such as `PUT`,
`PATCH`, or `DELETE` would couple UI composition artifacts to backend resource
models. It would also make experiment routing and policy enforcement harder to
reason about because every artifact could choose its own mutation semantics.

## Decision

Frontend submissions to the Composer use `POST` only.

The frontend-to-Composer contract for user intent submission is:

```text
POST page route
```

`PUT`, `PATCH`, `DELETE`, and other resource-oriented mutation verbs are not
part of the frontend artifact contract.

This decision applies to the public FE-to-Composer boundary. It does not forbid
backend or domain services from using resource-oriented HTTP methods internally.
The Composer may use separate backend integration policies, but UI artifacts
submit user intents to the Composer with `POST`.

Page configuration MAY distinguish `GET` and `POST` routes for the same path.
`GET` routes render pages. `POST` routes process submitted user intents through
Composer-owned routing and policy.

## Consequences

### Positive

- UI artifacts remain independent from backend resource models.
- Browser-native forms map naturally to the Composer submission contract.
- The Composer remains the policy gate for submitted user intents.
- Experiment-driven changes can replace submission handling without embedding
  branching logic in RFAs or IFAs.
- The public frontend contract stays small and easy to test.

### Negative / Risks

- Backend teams cannot expose arbitrary resource-oriented HTTP semantics
  directly through UI artifacts.
- Some workflows that look like resource updates must be modeled as submitted
  user intents.
- Additional Composer configuration is required to map a POST route to the
  backend service that processes the intent.

### Mitigations

- Treat POST submissions as intent messages and keep the backend resource model
  behind named services.
- Keep the Composer POST route configuration explicit and auditable.
- Use subsequent ADRs to define the redirect behavior and consistency model for
  POST submissions.
