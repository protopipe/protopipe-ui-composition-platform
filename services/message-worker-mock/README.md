# Message Worker Mock

Test-support service for broker-based interaction scenarios.

The service is intentionally technology-agnostic at the bounded-context level:
it represents a generic message worker mock. The first adapter consumes
RabbitMQ queues because RabbitMQ is the broker used for the next vertical slice.

## Responsibilities

- Consume configured message queues.
- Record every processed message in an in-memory processing log.
- Acknowledge messages only after they have been recorded.
- Expose the processing log over HTTP for Cucumber assertions.

## Non-Responsibilities

- It is not a product runtime service.
- It does not validate business semantics.
- It does not replace RabbitMQ; RabbitMQ is used for real broker semantics.

## Configuration

Environment variables:

- `MESSAGE_WORKER_MOCK_AMQP_URL`: RabbitMQ AMQP URL, defaults to `amqp://guest:guest@localhost:5672/%2f`
- `MESSAGE_WORKER_MOCK_QUEUES`: comma-separated queue names, defaults to `protopipe.commands`
- `MESSAGE_WORKER_MOCK_BINDINGS`: comma-separated `queue:exchange:routing-key` bindings, defaults to `protopipe.commands:protopipe.commands:#`
- `MESSAGE_WORKER_MOCK_HTTP_BIND`: HTTP bind address, defaults to `0.0.0.0:9100`

## HTTP API

- `GET /health`: readiness endpoint.
- `GET /processed`: returns all recorded processed messages.
- `DELETE /processed`: clears the processing log.
