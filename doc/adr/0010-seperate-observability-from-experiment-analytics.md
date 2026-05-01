# 0010 – Separate Observability from Experiment Analytics

Date: 2026-04-01

## Status

Accepted

## Context

The Protopipe UI Composition Platform requires telemetry for two fundamentally
different purposes:

1. **Observability**
   - system health
   - performance monitoring
   - error tracking
   - operational diagnostics

2. **Experiment Analytics**
   - user behavior analysis
   - experiment evaluation (e.g. conversion rates)
   - KPI measurement
   - decision support

These two concerns differ significantly in terms of:

- purpose
- data sensitivity
- legal requirements
- data retention and processing

Observability is required for the system to function reliably and securely,
while experiment analytics depends on user consent when it involves tracking
or identifying user behavior.

Mixing both concerns leads to:

- unclear data ownership
- legal risks (e.g. unintended tracking)
- architectural ambiguity
- difficulty in enforcing consent boundaries

A clear separation is required to ensure both compliance and system clarity.

Additionally, the platform must support:

- controlled data retention
- deletion of user-related data upon request
- irreversible anonymization of data where required

## Decision

Telemetry is strictly separated into two distinct categories:

### 1. Observability

Observability data is:

- collected for operational and technical purposes only
- independent of experiment logic
- not used to analyze user behavior across requests

Typical observability data includes:

- request counts
- response times
- error rates
- system-level metrics

Observability:

- MUST NOT include persistent user identifiers
- MUST NOT enable cross-session tracking
- MAY use short-lived or derived data strictly for operational needs
- SHOULD be aggregated as early as possible

### 2. Experiment Analytics

Experiment analytics is:

- used to evaluate experiments and user behavior
- dependent on explicit user consent
- based on identifiable or correlatable event data

Typical analytics data includes:

- experiment exposure events
- user interactions (e.g. clicks, conversions)
- session-based behavior

Experiment analytics:

- MUST only be collected if consent is given (see ADR-0009)
- MAY use pseudonymous identifiers
- MUST be clearly separated from observability pipelines

### 3. Data Lifecycle and Retention

The platform enforces explicit lifecycle rules for all telemetry data:

- Raw analytics data:
  - MUST have a defined retention period
  - SHOULD be deleted after it is no longer required

- User-related data:
  - MUST be deletable upon request
  - MUST NOT remain recoverable after deletion

- Aggregated data:
  - MAY be retained if it is fully anonymized
  - MUST NOT allow re-identification of individuals

- Anonymization:
  - MUST remove all identifiers and correlation capability
  - MUST be irreversible

- Observability data:
  - SHOULD be short-lived
  - SHOULD avoid storing identifiable information

## Consequences

### Positive

- Clear separation of responsibilities between system operation and product analytics
- Reduced legal and compliance risks
- Support for deletion and anonymization workflows
- Predictable data lifecycle management
- Improved clarity in data pipelines and system design

### Negative / Risks

- Increased architectural complexity due to dual telemetry pipelines
- Need to manage retention and deletion policies explicitly
- Potential loss of historical data due to enforced deletion

### Mitigations

- Provide clear guidelines and examples for telemetry usage
- Enforce separation through code structure and APIs
- Centralize analytics collection behind consent-aware interfaces
- Implement automated retention and deletion mechanisms
- Prefer aggregation over raw data storage where possible
