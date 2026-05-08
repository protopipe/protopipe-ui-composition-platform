Feature: Composer renders pages with RFA templates

  Scenario: Register a page and render it
    Given I register a page config:
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
      When I register an RFA "cart-rfa":
      """
      function(context) { return "Rendered: Currency is EUR"; }
      """
    And I request GET /my/shop/cart.fancy
    Then the response status should be 200
    And the response should contain "cart-page"
