Feature: Page routing
  As a product manager,
  I want to define page configurations for concrete and generic paths,
  so that the composer renders the matching page for each request.

  Rule: If I register a page config with a specific path, the composer should use that configuration when rendering the page for requests matching that path.

    Example: Register and render a page with a complex path
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

      Example: Register and render a page with a generic path
      Given a registered page config: 
        """
        {
          "path": "/my/shop/*",
          "page_id": "generic-shop-page",
          "type": "rfa",
          "template": "generic-shop-template",
          "rfa": "p_generic-shop-rfa_v1",
          "timeout_ms": 3000,
          "data": {
            "message": {
              "type": "static",
              "value": "Welcome to our shop"
            }
          }
        }
        """
      And a registered RFA "p_generic-shop-rfa_v1":
        """
        function(context) { return "Rendered: " + context.message; }
        """
      When I request GET /my/shop/some-category
      Then the response status should be 200
      And the response should contain "Rendered: Welcome to our shop"

      Example: Register and render a page with GET-Parameter wildcard
      Given a registered page config:
        """
        {
          "path": "/my/shop/search?query=*",
          "page_id": "search-page",
          "type": "rfa",
          "template": "search-template",
          "rfa": "search-rfa",
          "timeout_ms": 3000,
          "data": {
            "query": {
              "type": "getParameter",
              "key": "query"
            }
          }
        }
        """
      And a registered RFA "search-rfa":
        """
        function(context) { return "Rendered search results for query: " + context.query; }
        """
      When I request GET /my/shop/search?query=shoes
      Then the response status should be 200
      And the response should contain "Rendered search results for query: shoes"

  Rule: If I register a generic config it will be overwritten by more specific ones.

    Example: Register a generic and a specific page config
      Given a registered page config:
        """
        {
          "path": "/my/shop/*",
          "page_id": "generic-shop-page",
          "type": "rfa",
          "template": "generic-shop-template",
          "rfa": "generic-shop-rfa",
          "timeout_ms": 3000,
          "data": {
            "message": {
              "type": "static",
              "value": "Welcome to our shop"
            }
          }
        }
        """
      And a registered page config:
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
      And a registered RFA "generic-shop-rfa":
        """
        function(context) { return "Generic Rendered: " + context.message; }
        """
      And a registered RFA "cart-rfa":
        """
        function(context) { return "Cart Rendered: Currency is " + context.currency; }
        """
      When I request GET /my/shop/cart.fancy
      Then the response status should be 200
      And the response should contain "Cart Rendered: Currency is EUR"
