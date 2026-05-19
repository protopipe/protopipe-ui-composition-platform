Feature: Server-side page composition
  As a product manager,
  I want the composer to render pages based on registered page configurations and RFAs,
  so that pages are delivered as composed server-side responses.

  Rule: If I register a page config and an RFA, the composer should use them to render the page when requested.

    Example: Register a page and render it
      Given a registered page config:
        """
        {
          "path": "/my/shop/cart.fancy",
          "page_id": "cart-page",
          "type": "rfa",
          "template": "cart-v1",
          "rfa": "cart-rfa",
          "timeout_ms": 3000,
          "data": {
            "currency": {
              "type": "static",
              "value": "EUR"
            }
          }
        }
        """
      And a registered RFA "cart-rfa":
        """
        function(context) { return "Rendered: Currency is " + context.currency; }
        """
      When I request GET /my/shop/cart.fancy
      Then the response status should be 200
      And the response should contain "Rendered: Currency is EUR"
