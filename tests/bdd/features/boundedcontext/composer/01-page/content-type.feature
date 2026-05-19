Feature: Page response content type
  As a product manager,
  I want page configurations to control response content types,
  so that the composer can serve HTML and structured responses correctly.

  Rule: If the user specifies a content-type in the page config, the composer should use it in the response.

    Example: Default Content-Type is text/html
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
      And the response should have content-type "text/html; charset=utf-8"
      And the response should contain "Rendered: Currency is EUR"

    Example: Register a page config with content-type and render it
      Given a registered page config:
        """
        {
          "path": "/my/shop/cart.json",
          "page_id": "cart-page",
          "type": "rfa",
          "template": "cart-v1",
          "rfa": "cart-json-rfa",
          "timeout_ms": 3000,
          "content_type": "application/json",
          "data": {
            "currency": {
              "type": "static",
              "value": "EUR"
            }
          }
        }
        """
      And a registered RFA "cart-json-rfa":
        """
        function(context) { return JSON.stringify({ message: "Currency is " + context.currency }); }
        """
      When I request GET /my/shop/cart.json
      Then the response status should be 200
      And the response should have content-type "application/json"
      And the response should contain JSON:
        """
        {"message":"Currency is EUR"}
        """
