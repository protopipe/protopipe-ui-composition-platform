# 02 – Constraints

## Technical Constraints

- Kubernetes is the primary deployment and orchestration platform.
- Configuration must be declarative and resource-based (CRDs).
- All runtime components must be deployable and operable independently.
- The frontend platform must not require a shared frontend framework baseline.
- Rendering must support pure server-side execution (`template(data) -> html`).

## Organizational Constraints

- Development follows a GitLab-based workflow with feature branches and CI pipelines.
- Teams are autonomous and own their frontend fragments.
- Architecture must support dozens to hundreds of specialized frontend developers.
- Experiments are often owned by product or management roles, not engineering teams.

## Legal and Governance Constraints

- The project core must remain open and prevent proprietary forks.
- All improvements to the core must be eligible to flow back upstream.
- Monetization is expected through consulting, integration, training, and governance—not
  licensing fees.

