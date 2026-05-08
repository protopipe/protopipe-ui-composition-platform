#!/bin/bash

# Quick test script for local development

echo "Starting Composer Service..."
cargo run &
COMPOSER_PID=$!

# Give the service time to start
sleep 3

echo ""
echo "Testing Admin Health Check..."
curl -s http://localhost:9000/admin/health | jq .
echo ""

echo ""
echo "Registering page config..."
curl -s -X POST http://localhost:9000/admin/config/pages \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/shop/cart",
    "page_id": "cart-page",
    "template": "cart-v1",
    "rfa": "cart-rfa",
    "timeout_ms": 3000,
    "defaults": {"currency": "EUR"}
  }' | jq .

echo ""
echo "Registering RFA..."
curl -s -X POST http://localhost:9000/admin/rfa/register \
  -H "Content-Type: application/json" \
  -d '{
    "id": "cart-rfa",
    "source": "function render() { return \"Cart Template\"; }",
    "version": "1.0.0"
  }' | jq .

echo ""
echo "Rendering page..."
curl -s http://localhost:8080/shop/cart
echo ""

# Cleanup
kill $COMPOSER_PID
