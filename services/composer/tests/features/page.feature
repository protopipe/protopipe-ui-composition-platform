Feature: Page Configuration
As a product manager, I want to be able to define page configurations in the composer, so that I can specify how pages should be rendered and what data they should use.
It is important that I can define complex Paths and Parameters for my pages, so that I can create a rich user experience.

Rule: If I register a page config with a specific path, the composer should use that configuration when rendering the page for requests matching that path.
Example: Register and render a page with a complex path
    Given a registered page config:
      """
      {
        "path": "/my/shop/cart.fancy",
        "page_id": "cart-page",
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


Rule: If I register a generic config it will be overwritten by more specific ones
Example: Register a generic and a specific page config
    Given a registered page config:
      """
      {
        "path": "/my/shop/*",
        "page_id": "generic-shop-page",
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

    Rule: If the user specifies a content-type in the page config, the composer should use it in the response.

        Example: Default Content-Type is text/html
        Given a registered page config:
        """
        {
            "path": "/my/shop/cart.fancy",
            "page_id": "cart-page",
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

        @WIP
        Example: Register a page config with content-type and render it
        Given a registered page config:
        """
        {
            "path": "/my/shop/cart.json",
            "page_id": "cart-page",
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
        And the response should contain '{"message":"Currency is EUR"}'

      