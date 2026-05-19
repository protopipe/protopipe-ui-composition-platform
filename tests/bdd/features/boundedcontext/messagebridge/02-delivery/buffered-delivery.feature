Feature: Message Bridge buffered polling-based delivery
  As a platform operator,
  I want interaction events to be delivered through buffered polling,
  so that clients can tolerate offline periods without requiring push sockets.

  Rule: Client-side and server-side message bridges may temporarily buffer interaction events.

  Rule: Clients retrieve pending events by polling the message endpoint.

  Rule: Message bridges preserve event identifiers and correlation identifiers across delivery.
