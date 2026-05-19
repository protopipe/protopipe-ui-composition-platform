Feature: Message Bridge interaction event lifecycle
  As a UI artifact owner,
  I want interaction events to move through explicit lifecycle states,
  so that optimistic UI behavior can be reconciled predictably.

  Rule: Interaction events move from observed to buffered, delivered, processed, and resolved.

    Example: Frontend interaction event is delivered to a worker through RabbitMQ
      Given a registered IFA message channel:
        """
        {
          "ifa_id": "cookie-banner-ifa",
          "events": {
            "cookie.consent.confirmed": {
              "exchange": "protopipe.commands",
              "routing_key": "cookie.consent.confirmed"
            }
          }
        }
        """
      When the frontend emits an interaction event:
        """
        {
          "message_id": "msg-cookie-consent-1",
          "correlation_id": "corr-cookie-consent-1",
          "ifa_id": "cookie-banner-ifa",
          "name": "cookie.consent.confirmed",
          "context": {
            "page_id": "cookie-demo",
            "artifact_id": "cookie-banner-ifa",
            "artifact_type": "ifa"
          },
          "payload": {
            "confirmed": true
          }
        }
        """
      Then the response status should be 202
      And the worker mock should have processed message "cookie.consent.confirmed" on queue "protopipe.commands"

  Rule: Delivered interaction events are not authoritative until they are processed and resolved.

  Rule: Technical failures are represented separately from business events.
