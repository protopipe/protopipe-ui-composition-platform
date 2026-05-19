Feature: Message Bridge governed business event delivery
  As a platform operator,
  I want business events to be delivered with field-level governance,
  so that consumers can observe event sequences without receiving data they are not allowed to see.

  Rule: Business event schemas declare the sensitivity of every payload property.

  Rule: Business events carry the state required by their intended consumers.

  Rule: Consumer registration defines trust, scopes, batch limits, and replay permissions.

  Rule: Field-level redaction preserves cursor continuity.

    Example: Consumer receives an empty payload for an event whose fields are not visible
      Given a business event schema "customer.address.changed" with classified payload fields
      And a registered consumer "invoice-service" without permission to read customer address fields
      When business event "customer.address.changed" at cursor "customer-events:42" is delivered to "invoice-service"
      Then the delivered business event should keep cursor "customer-events:42"
      And the delivered business event payload should be empty
      And the delivered business event should keep its type, subject, and partition metadata

    Example: Authorized consumer receives classified state without follow-up readback
      Given a business event schema "customer.address.changed" with classified payload fields
      And a registered consumer "shipping-service" with permission to read customer address fields
      When business event "customer.address.changed" at cursor "customer-events:43" is delivered to "shipping-service"
      Then the delivered business event should contain the address state required by "shipping-service"
      And "shipping-service" should not need to read the address from the domain service to interpret the event

  Rule: Business event sourcing is optional per stream.

  Rule: Business event replay is available only for sourced streams with retention.

  Rule: Shadow and experiment services synchronize from business-object event streams.

    Example: Shadow service reaches the active service cursor before activation
      Given a business-object stream "customer-events"
      And an active consumer "customer-service"
      And a shadow consumer "customer-service-shadow"
      When business events up to cursor "customer-events:120" are delivered to both consumers
      Then "customer-service-shadow" should have committed cursor "customer-events:120"
      And "customer-service-shadow" should be eligible for experiment traffic activation

  Rule: Experiment complexity remains outside service implementation.

    Example: Experiment service receives the same business-object events through registration
      Given a business-object stream "customer-events"
      And experiment service "customer-service-v2" is registered as a consumer
      When business event "customer.updated" at cursor "customer-events:121" is delivered
      Then "customer-service-v2" should receive the event through the Message Bridge
      And "customer-service-v2" should not need experiment assignment logic to synchronize its local state
