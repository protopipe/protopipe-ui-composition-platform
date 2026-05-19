Feature: Message Bridge webhook batch delivery
  As a service owner,
  I want to receive message batches over HTTP,
  so that I can consume Protopipe messages without depending on broker-specific protocols.

  Rule: Message Bridges deliver batches to registered webhook consumers.

    Example: Consumer acknowledges a delivered batch
      Given a registered webhook consumer "invoice-service" for stream "customer-events"
      And the consumer "invoice-service" accepts batches with at most 100 messages
      When the Message Bridge delivers messages from cursor "customer-events:41" to cursor "customer-events:45"
      And the consumer acknowledges the batch
      Then the Message Bridge should commit cursor "customer-events:45" for consumer "invoice-service"

  Rule: Unacknowledged batches are retried.

  Rule: Consumers are responsible for semantic idempotency.

  Rule: Consumers may request replay from one cursor position to another when the stream supports replay.

    Example: Consumer requests replay for a missed cursor range
      Given stream "customer-events" supports business event sourcing
      And consumer "invoice-service" is allowed to replay stream "customer-events"
      When consumer "invoice-service" requests replay from cursor "customer-events:41" to cursor "customer-events:45"
      Then the Message Bridge should redeliver the configured cursor range to the consumer webhook

  Rule: Message Bridges may be chained through the same webhook batch protocol.

    Example: Edge bridge buffers a batch before forwarding it upstream
      Given a downstream Message Bridge "store-bridge-17"
      And an upstream Message Bridge "central-bridge"
      When "store-bridge-17" receives a batch while the upstream bridge is unavailable
      Then "store-bridge-17" should buffer the batch durably
      And "store-bridge-17" should forward the batch when "central-bridge" becomes reachable

  Rule: Delivery identifiers are local to the bridge layer that exposes them.

    Example: Consumer receives edge-local cursors from an edge bridge
      Given consumer "store-dashboard" is registered at Message Bridge "store-bridge-17"
      And Message Bridge "store-bridge-17" receives upstream event "central:customer-events:45"
      When "store-bridge-17" delivers the event to consumer "store-dashboard"
      Then the delivered event should contain a cursor local to "store-bridge-17"
      And the delivered event may include upstream cursor "central:customer-events:45" as metadata
      And consumer "store-dashboard" should not use the upstream cursor as its committed position
