Feature: Partial Rendering


Rule: The composer should support partial rendering of page sections based on registered RFAs and page configs.

  Example: Render a page section with an RFA
    Given a registered page config:
      """
      {
        "path": "/my/shop/cart.partial",
        "page_id": "cart-page",
        "type": "rfa",
        "template": "cart-v1",
        "rfa": "p_shop_v1",
        "timeout_ms": 3000,
        "data": {
          "currency": {
            "type": "static",
            "value": "USD"
          }
        }
      }
      """
    And a registered RFA "p_shop_v1":
        """
        function(context, partials) { return partials.include("o_cart_v1", context); }
        """
    And a registered RFA "o_cart_v1":
      """
      function(context) { return "Partial Rendered: Currency is " + context.currency; }
      """
    When I request GET /my/shop/cart.partial
    Then the response status should be 200
    And the response should contain "Partial Rendered: Currency is USD"