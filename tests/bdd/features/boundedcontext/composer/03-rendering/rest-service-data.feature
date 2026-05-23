Feature: REST service data
  As a product manager,
  I want page configuration to declare REST-backed data values,
  so that RFAs receive resolved backend data without performing service calls.

  Rule: REST service data is resolved by the Composer before RFA execution.

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

    Scenario: RFA receives runtime data resolved from the backend before execution

      Given a backend service "catalog"
      And backend service "catalog" returns templated JSON for GET /products/sku-123:
        """
        {
          "name": "Trail Shoe",
          "resolvedAt": "{{now format='unix'}}"
        }
        """
      And a registered page config:
        """
        {
          "path": "/product-runtime.html",
          "page_id": "product-runtime",
          "type": "rfa",
          "template": "product",
          "rfa": "p_product_runtime_v1",
          "timeout_ms": 3000,
          "data": {
            "product": {
              "type": "restService",
              "service": "catalog",
              "path": "/products/sku-123",
              "method": "GET",
              "timeout_ms": 250,
              "error_default": {
                "name": "Unknown product",
                "resolvedAt": "missing"
              }
            }
          }
        }
        """
      And a registered RFA "p_product_runtime_v1":
        """
        function(context) { return context.product.name + " resolvedAt=" + context.product.resolvedAt + " createdAt=" + (Math.floor(Date.now() / 1000) + 1); }
        """
      When I request GET /product-runtime.html
      Then the response should contain "Trail Shoe resolvedAt="
      And extract from response "resolvedAt=(\d+)" as resolvedAt
      And extract from response "createdAt=(\d+)" as createdAt
      And assert that resolvedAt is less than createdAt
      And the response should contain a unix timestamp after "resolvedAt="
      And backend service "catalog" should have received 1 request for GET /products/sku-123

  Rule: Multiple REST service data values are resolved concurrently within the request budget.

  Rule: REST service data is resolved after experiment overrides have produced the effective page configuration.

  Rule: REST service calls must use bounded timeouts and explicit degraded rendering behavior.
