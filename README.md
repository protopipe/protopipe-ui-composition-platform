# Protopipe Frontend Platform

Kubernetes-native frontend platform for SSR-based UI composition,
artifact-driven experimentation, and contract-first development.

## Architecture Documentation (arc42)

This repository uses the **arc42 architecture documentation template**.
The README serves as an index; detailed architecture documentation lives in
dedicated chapters under `docs/arc42`.

👉 **Start here:**
- [Architecture Overview](doc/arc42/README.md)


## Repository Structure

- `services/` – runtime components (operator, composer, message bridge)
- `packages/` – shared specs, ABIs, tooling
- `docs/` – architecture, ADRs, diagrams
- `deployments/` – Helm / Kustomize
- `examples/` – reference implementations
- `tests/bdd/` – Cucumber blackbox tests
- `tests/load/` – k6 smoke load tests

## Development

The local and CI execution model is based on the Compose Specification. The
same profiles are used locally and in GitHub Actions.

```bash
podman compose --profile dev run --rm bdd-dev
```

`bdd-dev` is configured so you can pass additional Cucumber args/tags on
the command line.

When you are done, stop services started by Compose:

```bash
podman compose --profile dev down --remove-orphans
```

Run blackbox BDD tests:

```bash
podman compose --profile test up --build --abort-on-container-exit --exit-code-from test-bdd
```

Run the k6 smoke load test:

```bash
podman compose --profile load up --build --abort-on-container-exit --exit-code-from test-load
```

## License

This project is licensed under **GPL-3.0-only**.  
See [LICENSE](LICENSE) for details.
