# Message Bridge

Minimal Message Bridge service for the interaction delivery vertical slice.

The service accepts client-side interaction events over HTTP, resolves the
configured IFA publish channel, and publishes the message to RabbitMQ. It does
not perform experiment assignment and does not own business semantics.

## HTTP API

- `GET /health`
- `POST /admin/config/ifas`
- `DELETE /admin/config`
- `POST /messages`

## Environment

- `MESSAGEBRIDGE_AMQP_URL`: RabbitMQ AMQP URL, defaults to `amqp://guest:guest@localhost:5672/%2f`
- `MESSAGEBRIDGE_HTTP_BIND`: HTTP bind address, defaults to `0.0.0.0:8082`
