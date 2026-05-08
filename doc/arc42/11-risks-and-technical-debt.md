# 11 – Risks and Technical Debt

## Key Risks

### 1. Platform Complexity Concentration

**Risk**: The Composer service and Frontend Operator become single points of failure and bottlenecks for understanding the entire system.

**Impact**: High (affects all rendered pages)

**Mitigation**:
- Clear separation of concerns (events, rendering, composition)
- Well-documented contracts and interfaces (RFA ABI, event contracts)
- Extensive testing and monitoring
- Operator as open-source reference implementation

### 2. Snapshot Artifact Sprawl

**Risk**: Uncontrolled growth of snapshot artifacts from experiments, occupying storage and complicating cleanup.

**Impact**: Medium (operational cost and complexity)

**Mitigation**:
- Define artifact retention policies per experiment
- Automatic garbage collection based on age and usage
- Tagging and metadata for lifecycle management
- Regular cleanup jobs

### 3. Experiment Interference

**Risk**: Multiple concurrent experiments create complex interaction effects, making results ambiguous.

**Impact**: Medium-High (affects experiment validity)

**Mitigation**:
- Clear experiment isolation rules
- Mandatory exclusions between conflicting experiments
- Proper statistical design (see [ADR-0008](../adr/0008-centralize-experiment-routing-before-rendering.md))
- Centralized routing prevents unintended interactions (see [ADR-0008](../adr/0008-centralize-experiment-routing-before-rendering.md))

### 4. Observability Drift Across Technologies

**Risk**: Different RFA implementations (Node.js, Python, Go, etc.) use inconsistent telemetry, making system-wide observability unreliable.

**Impact**: Medium (affects debugging and monitoring)

**Mitigation**:
- Standardized telemetry hooks in RFA ABI
- Sidecar-based metric collection per artifact
- Enforced logging formats and correlation IDs
- Validation in CDCT for observability contracts

### 5. SSR Performance Bottlenecks

**Risk**: Server-side composition becomes too slow as complexity grows, degrading time-to-first-byte.

**Impact**: High (affects user experience)

**Mitigation** (see [ADR-0007](../adr/0007-ssr-first-rendering-model.md)):
- Explicit composition latency budgets (e.g., 200ms)
- Parallel RFA execution
- Caching of resolved templates, assignments, and RFA outputs
- Graceful degradation (fallback rendering, timeout strategies)
- Load testing and production monitoring

### 6. Privacy Compliance Drift

**Risk**: Observability and analytics pipelines become entangled again, re-introducing privacy risks.

**Impact**: High (legal and reputational risk)

**Mitigation** (see [ADR-0010](../adr/0010-seperate-observability-from-experiment-analytics.md)):
- Strict architectural separation in code and deployment
- Automated validation that pipelines remain separate
- Regular compliance audits
- Clear data retention and deletion policies

### 7. Event Lifecycle Complexity

**Risk**: Offline clients, buffering, deferred delivery, and retries create complex state reconciliation challenges.

**Impact**: Medium (affects correctness of interactive features)

**Mitigation** (see [ADR-0012](../adr/0012-define-interaction-event-lifecyle.md)):
- Defined event lifecycle (Observed → Buffered → Delivered → Processed → Resolved)
- Idempotent event handlers
- Clear correlation IDs for tracing
- Comprehensive error handling and logging
- Testing against failure scenarios

## Technical Debt

### 1. RFA Isolation

**Status**: Design-complete, implementation-in-progress

- RFAs currently run in-process with the Composer
- Future: Implement isolated execution (containers, VMs, sandboxes)
- Trade-off: Performance vs isolation
- Decision required: How strict to make isolation (in-process threads → processes → containers)

### 2. Event Delivery Semantics

**Status**: Design-complete (see [ADR-0014](../adr/0014-use-buffered-polling-based-event-delivery.md)), implementation-in-progress

- Current: Best-effort delivery via polling
- Future: Exactly-once semantics may be required for financial transactions
- Trade-off: Simplicity vs correctness guarantees
- Investigation needed: Cost of exactly-once without traditional message queues

### 3. Template and Layout Versioning

**Status**: Open question

- How to evolve Templates without breaking RFAs?
- How to handle backwards-compatibility?
- Migration paths for deprecated Slots?
- Schema versioning for context and data contracts?

### 4. Artifact Caching Strategy

**Status**: Partially implemented

- Current: Simple TTL-based caching
- Future: Invalidation strategies based on dependency changes
- Question: When should cached RFA output be invalidated?
  - On experiment change?
  - On template change?
  - On backend data change?

### 5. Cross-cutting Observability

**Status**: Design-complete, standardization-in-progress

- Each team implements observability independently
- Risk: Inconsistent metrics, missing correlation IDs
- Future: Centralized sidecar for metric collection, trace propagation

## Mitigations and Monitoring

### Short-term (next 2-3 releases)

- [ ] Document RFA isolation requirements and design options
- [ ] Implement artifact retention and garbage collection
- [ ] Add experiment interference detection (statistical analysis)
- [ ] Standardize telemetry hooks across reference RFAs

### Medium-term (next 6 months)

- [ ] Implement RFA isolation (process boundaries)
- [ ] Design template versioning strategy
- [ ] Add exactly-once event delivery option
- [ ] Centralize observability collection sidecar

### Long-term (roadmap)

- [ ] Evaluate strict RFA sandboxing (VMs, containers)
- [ ] Support fine-grained experiment interactions (multi-armed bandit algorithms)
- [ ] Advanced caching with dependency graphs
- [ ] Continuous compliance monitoring for privacy regulations
