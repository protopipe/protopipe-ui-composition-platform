# Composer Service

Minimal Composer Service for Protopipe UI Composition Platform.

Built with **Actix-web** for maximum performance and non-blocking async I/O.

## Quick Start

### Local Development

```bash
# Build and run locally
cargo run

# In another terminal, test the service
curl -X POST http://localhost:9000/admin/config/pages \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/my/page",
    "page_id": "test-page",
    "template": "test-template",
    "rfa": "test-rfa",
    "timeout_ms": 3000,
    "defaults": {}
  }'

curl http://localhost:8080/my/page
```

### Docker Compose

```bash
# Start all services (Composer + WireMock)
docker-compose up --build

# Test
curl -X POST http://localhost:9000/admin/config/pages \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/shop/cart",
    "page_id": "cart-page",
    "template": "cart-v1",
    "rfa": "cart-rfa",
    "timeout_ms": 3000,
    "defaults": {}
  }'

curl http://localhost:8080/shop/cart
```

## Ports

- **Admin Port (9000)**: `/admin/config/pages`, `/admin/rfa/register`, `/admin/health`
- **Render Port (8080)**: Dynamic page rendering (catch-all)

## Benchmarking

A k6 benchmark is available under `services/composer/bench/k6/benchmark.js`.
The GitHub Actions workflow is defined in `.github/workflows/composer-k6-benchmark.yml`.
You can test Github Actions with the Github-Extension act:
```
 gh extension install https://github.com/nektos/gh-act
```
and run via:
```
 gh act workflow_dispatch -j composer-benchmark
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
