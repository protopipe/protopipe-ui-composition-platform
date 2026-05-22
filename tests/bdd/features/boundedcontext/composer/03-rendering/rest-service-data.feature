Feature: REST service data
  As a product manager,
  I want page configuration to declare REST-backed data values,
  so that RFAs receive resolved backend data without performing service calls.

  Rule: REST service data is resolved by the Composer before RFA execution.

  Rule: Multiple REST service data values are resolved concurrently within the request budget.

  Rule: REST service data is resolved after experiment overrides have produced the effective page configuration.

  Rule: REST service calls must use bounded timeouts and explicit degraded rendering behavior.

  Scenario: RFA receives data returned by a configured REST service

    Given a backend service "catalog"
    And backend service "catalog" returns JSON for GET /products/sku-123:
      """
      {
        "name": "Trail Shoe"
      }
      """
    And a registered page config:
      """
      {
        "path": "/product.html",
        "page_id": "product",
        "type": "rfa",
        "template": "product",
        "rfa": "p_product_v1",
        "timeout_ms": 3000,
        "data": {
          "product": {
            "type": "restService",
            "service": "catalog",
            "path": "/products/sku-123",
            "method": "GET",
            "timeout_ms": 250,
            "error_default": {
              "name": "Unknown product"
            }
          }
        }
      }
      """
    And a registered RFA "p_product_v1":
      """
      function(context) { return context.product.name; }
      """
    When I request GET /product.html
    Then the response should contain "Trail Shoe"
