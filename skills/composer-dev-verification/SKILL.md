---
name: composer-dev-verification
description: Use in this repository when verifying the Composer dev setup, checking Rust compiler errors from the composer-dev container logs, or running BDD/Cucumber scenarios through compose instead of invoking local cargo directly.
---

# Composer Dev Verification

Use this skill when work on the Composer service needs validation through the
repository dev setup.

## Workflow

1. Prefer the compose dev stack over direct local service execution.
2. Check whether `composer-dev` is already running before starting anything.
3. Read Rust compiler/build errors from the `composer-dev` container logs.
4. Run Cucumber through compose services, not via direct local `cargo test`,
   unless the user explicitly asks for local execution.

## Commands

Use `podman-compose` from the repository root unless the environment clearly
uses Docker Compose.

Check running services:

```bash
podman-compose -f compose.yaml --profile dev ps
```

Start or refresh the dev Composer stack:

```bash
podman-compose -f compose.yaml --profile dev up -d composer-dev wiremock
```

Inspect compiler/build feedback from the Composer dev container:

```bash
podman-compose -f compose.yaml --profile dev logs --tail=200 composer-dev
```

Follow logs while waiting for bacon to rebuild:

```bash
podman-compose -f compose.yaml --profile dev logs -f composer-dev
```

Run WIP Cucumber scenarios through compose:

```bash
podman-compose -f compose.yaml --profile dev run --rm bdd-dev
```

`bdd-dev` tests only examples/scenarios tagged with `@WIP` by default, because
the compose service passes `--tags @WIP`.

Run the normal non-WIP BDD suite:

```bash
podman-compose -f compose.yaml --profile test run --rm test-bdd
```

## Interpretation

- Treat `composer-dev` logs as the source of truth for Rust compiler errors in
  the dev setup, because the service uses the mounted Composer source and
  container-local target/cache volumes.
- If ports `8080` or `9000` are already occupied by `rootlessport`, assume the
  compose stack may already be running and inspect compose state/logs first.
- If compose commands fail because the runtime is unavailable or permissions are
  missing, report that clearly and fall back only to `cargo fmt` / `cargo check`
  for local static validation.
- For Cucumber feature selection, prefer the existing compose service defaults:
  `bdd-dev` runs only `@WIP`, while `test-bdd` runs `not @WIP`.

## Notes

- The Composer admin endpoint is `http://localhost:9000/admin/health`.
- The render endpoint listens on `http://localhost:8080`.
- The dev compose setup maps `composer-dev` to both ports and mounts
  `./services/composer:/app`.
