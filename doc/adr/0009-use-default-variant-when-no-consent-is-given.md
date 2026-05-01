# 0009 – Use Default Variant When No Consent is Given

Date: 2026-04-01

## Status

Accepted

## Context

The Protopipe UI Composition Platform relies on experiment-driven delivery
to optimize user interfaces based on measurable outcomes.

At the same time, the platform must operate in environments subject to
privacy regulations such as the :contentReference[oaicite:0]{index=0} (GDPR),
which restrict the use of tracking mechanisms without explicit user consent.

Experiment assignment and analytics typically require:

- persistent identifiers (e.g. cookies)
- cross-request correlation
- behavioral tracking

Without user consent, these mechanisms cannot be used.

The system must therefore define a consistent behavior for users who do not
provide consent, while maintaining:

- functional correctness
- predictable rendering
- architectural integrity

## Decision

If no consent is given, the system will:

- assign the user to the **default (control) variant**
- avoid any form of persistent experiment assignment
- avoid any tracking or analytics that requires user identification

This implies:

- Experiment routing still occurs (see ADR-0008), but:
  - only the default variant is selected
  - no variant diversification is applied

- No identifiers are stored for:
  - user tracking
  - experiment participation
  - cross-session correlation

- The system does not attempt to:
  - infer identity
  - reconstruct sessions
  - simulate tracking via alternative mechanisms (e.g. fingerprinting)

- Rendering remains fully functional and deterministic

## Consequences

### Positive

- Compliance with privacy regulations without degrading functionality
- Clear and predictable behavior for non-consenting users
- Strong separation between consent-based and non-consent-based execution paths
- Reduced legal and reputational risk

### Negative / Risks

- Reduced ability to measure experiment impact for non-consenting users
- Potential bias in analytics due to partial visibility
- Loss of personalization or optimization for a subset of users

### Mitigations

- Use aggregated, non-identifying metrics where permissible
- Clearly separate:
  - observability data
  - experiment analytics
- Design experiments to tolerate partial data
- Communicate transparently about the impact of consent on optimization
