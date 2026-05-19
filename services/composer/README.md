# Composer Service

Minimal Composer Service for Protopipe UI Composition Platform.

Built with **Actix-web** for maximum performance and non-blocking async I/O.

## Quick Start

### Local Development

```bash
# Build and run locally with bacon
RUST_LOG=DEBUG bacon long_run

# Or run through the project Compose environment (auto-starts dependencies)
podman compose -f ../../compose.yaml --profile dev run --rm bdd-dev

# Stop started services when done
podman compose -f ../../compose.yaml --profile dev down --remove-orphans

# In another terminal, test the service
curl -X POST http://localhost:9000/admin/config/pages \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/my/page",
    "page_id": "test-page",
    "type": "rfa",
    "template": "test-template",
    "rfa": "test-rfa",
    "timeout_ms": 3000,
    "defaults": {}
  }'

curl http://localhost:8080/my/page
```

### Testing

The Cucumber tests are blackbox tests under `tests/bdd` at the repository root.
Run them through the shared Compose environment:

```bash
podman compose -f ../../compose.yaml --profile test up --build --abort-on-container-exit --exit-code-from test-bdd
```

For ad-hoc retry during local development, use the `dev` profile:

```bash
podman compose -f ../../compose.yaml --profile dev run --rm bdd-dev
```

### Docker Compose

```bash
# Run Cucumber in dev mode (Composer + WireMock are auto-started)
podman compose -f ../../compose.yaml --profile dev run --rm bdd-dev

# Stop started services
podman compose -f ../../compose.yaml --profile dev down --remove-orphans

# Test
curl -X POST http://localhost:9000/admin/config/pages \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/shop/cart",
    "page_id": "cart-page",
    "type": "rfa",
    "template": "cart-v1",
    "rfa": "cart-rfa",
    "timeout_ms": 3000,
    "defaults": {}
  }'

curl http://localhost:8080/shop/cart
```

### GitHub Actions

The CI workflow uses Docker Compose for service startup, tests, and the k6
smoke test. GitHub Actions is intentionally thin and invokes the same
`compose.yaml` profiles used locally.

## Ports

- **Admin Port (9000)**: `/admin/config/pages`, `/admin/rfa/register`, `/admin/health`
- **Render Port (8080)**: Dynamic page rendering (catch-all)

## Benchmarking

A k6 smoke test is available under `tests/load/composer-smoke.js`.
Run it locally with:

```
podman compose -f ../../compose.yaml --profile load up --build --abort-on-container-exit --exit-code-from test-load
```


## Architecture Components

- `admin.rs`: Admin endpoint handlers (async/await)
- `render.rs`: Dynamic page rendering with catch-all routing
- `main.rs`: Dual HTTP server bootstrap with independent async runtimes

## Non-blocking Architecture

- ✓ Dual concurrent `HttpServer` instances
- ✓ Full async/await with Tokio runtime
- ✓ No socket blocking or thread pools
- ✓ Ready for Futures-based external service calls
- ✓ Suitable for RFA sandboxing

## Next Steps

1. Integrate Deno/JS sandbox for RFA execution
2. Add external service adapter with futures/timeouts
3. Implement health checks & observability
