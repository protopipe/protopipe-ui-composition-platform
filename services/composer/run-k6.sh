#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BENCH_DIR="$SCRIPT_DIR/target/bench"

mkdir -p "$BENCH_DIR"

echo -e "${YELLOW}[k6 Benchmark]${NC} Starting Composer service and k6 benchmark..."

# Start docker-compose in the background
cd "$PROJECT_ROOT"
docker-compose up -d composer wiremock

# Wait for Composer to be ready
echo -e "${YELLOW}[k6 Benchmark]${NC} Waiting for Composer health check..."
for i in $(seq 1 30); do
  if curl -fs http://127.0.0.1:9000/admin/health >/dev/null 2>&1; then
    echo -e "${GREEN}[k6 Benchmark]${NC} Composer is ready"
    break
  fi
  if [ $i -eq 30 ]; then
    echo -e "${RED}[k6 Benchmark]${NC} Composer did not start in time"
    docker-compose logs composer
    docker-compose down
    exit 1
  fi
  sleep 1
done

# Run k6 benchmark with output to target/bench
echo -e "${YELLOW}[k6 Benchmark]${NC} Running k6 benchmark..."
DOCKER_RUNTIME=${ACT_CONTAINER_RUNTIME:-docker}

$DOCKER_RUNTIME run --rm \
  -v "$SCRIPT_DIR:/work" \
  -w /work \
  --network host \
  grafana/k6:latest \
  run \
    --summary-export=/work/target/bench/k6-summary.json \
    --out json=/work/target/bench/k6-results.json \
    bench/k6/benchmark.js

# Cleanup
echo -e "${YELLOW}[k6 Benchmark]${NC} Stopping services..."
docker-compose down

if [ -f "$BENCH_DIR/k6-summary.json" ]; then
  echo -e "${GREEN}[k6 Benchmark]${NC} ✅ Benchmark complete"
  echo -e "${GREEN}[k6 Benchmark]${NC} Results saved to:"
  echo "  - $BENCH_DIR/k6-summary.json"
  echo "  - $BENCH_DIR/k6-results.json"
else
  echo -e "${RED}[k6 Benchmark]${NC} ❌ Benchmark failed"
  exit 1
fi
