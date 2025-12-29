# 03 – System Context

## Business Context

The platform sits between:

- End users requesting web pages,
- Backend services providing domain data,
- Product and management roles defining experiments and KPIs.

It acts as a **frontend composition and experimentation layer**, coordinating rendering,
backend access, and observability.

## Technical Context

External systems include:

- Kubernetes API server (CRDs, status reconciliation),
- CI/CD pipelines publishing snapshot artifacts,
- Backend services providing domain data,
- Central observability and tracing infrastructure.

The frontend platform itself consists of:

- A **Composer** for SSR and composition,
- A **Frontend Operator** for Page and Experiment resources,
- An optional **Message Bridge** for reactive UI updates.

