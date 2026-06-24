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

    Scenario: REST service request query parameters are mapped explicitly from the Composer request

      Given a backend service "catalog"
      And backend service "catalog" returns JSON for GET /products?produktNr=sku-123:
        """
        {
          "produktNr": "sku-123",
          "name": "Trail Shoe"
        }
        """
      And a registered page config:
        """
        {
          "path": "/product-by-number.html",
          "page_id": "product-by-number",
          "type": "rfa",
          "template": "product",
          "rfa": "p_product_by_number_v1",
          "timeout_ms": 3000,
          "data": {
            "product": {
              "type": "restService",
              "service": "catalog",
              "path": "/products",
              "method": "GET",
              "timeout_ms": 250,
              "request": {
                "query": {
                  "produktNr": {
                    "from": "query",
                    "name": "produktNr"
                  }
                }
              },
              "error_default": {
                "name": "Unknown product"
              }
            }
          }
        }
        """
      And a registered RFA "p_product_by_number_v1":
        """
        function(context) { return context.product.produktNr + " " + context.product.name; }
        """
      When I request GET /product-by-number.html?produktNr=sku-123
      Then the response should contain "sku-123 Trail Shoe"
      And backend service "catalog" should have received 1 request for GET /products?produktNr=sku-123

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

    Scenario: Independent REST service data values are resolved concurrently

      Given a backend service "catalog"
      And backend service "catalog" returns templated JSON after 700 ms for GET /products/sku-123:
        """
        {
          "name": "Trail Shoe",
          "resolvedAt": "{{now format='unix'}}"
        }
        """
      And backend service "catalog" returns templated JSON after 700 ms for GET /prices/sku-123:
        """
        {
          "amount": 129,
          "currency": "EUR",
          "resolvedAt": "{{now format='unix'}}"
        }
        """
      And a registered page config:
        """
        {
          "path": "/product-composed.html",
          "page_id": "product-composed",
          "type": "rfa",
          "template": "product",
          "rfa": "p_product_composed_v1",
          "timeout_ms": 3000,
          "data": {
            "product": {
              "type": "restService",
              "service": "catalog",
              "path": "/products/sku-123",
              "method": "GET",
              "timeout_ms": 1500,
              "error_default": {
                "name": "Unknown product"
              }
            },
            "price": {
              "type": "restService",
              "service": "catalog",
              "path": "/prices/sku-123",
              "method": "GET",
              "timeout_ms": 1500,
              "error_default": {
                "amount": 0,
                "currency": "EUR"
              }
            }
          }
        }
        """
      And a registered RFA "p_product_composed_v1":
        """
        function(context) { return context.product.name + " costs " + context.price.amount + " " + context.price.currency; }
        """
      When I request GET /product-composed.html
      Then the response should contain "Trail Shoe costs 129 EUR"
      And the last request should complete within 1300 ms
      And backend service "catalog" should have received 1 request for GET /products/sku-123
      And backend service "catalog" should have received 1 request for GET /prices/sku-123

  Rule: REST service data is resolved after experiment overrides have produced the effective page configuration.

    Scenario: Experiment data override replaces the REST service before data resolution

      Given a backend service "catalog"
      And a backend service "catalog-preview"
      And backend service "catalog" returns JSON for GET /products/sku-123:
        """
        {
          "name": "Trail Shoe"
        }
        """
      And backend service "catalog-preview" returns JSON for GET /products/sku-123:
        """
        {
          "name": "Preview Trail Shoe"
        }
        """
      And a registered page config:
        """
        {
          "path": "/product-experiment.html",
          "page_id": "product-experiment",
          "type": "rfa",
          "template": "product",
          "rfa": "p_product_experiment_v1",
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
      And a registered experiment:
        """
        {
          "experiment_id": "product_data_source_test",
          "scope": {
            "path": "/product-experiment.html"
          },
          "variants": [
            {
              "id": "control",
              "weight": 50
            },
            {
              "id": "preview",
              "weight": 50,
              "overrides": {
                "data": {
                  "product": {
                    "type": "restService",
                    "service": "catalog-preview",
                    "path": "/products/sku-123",
                    "method": "GET",
                    "timeout_ms": 250,
                    "error_default": {
                      "name": "Unknown product"
                    }
                  }
                }
              }
            }
          ]
        }
        """
      And I have accepted the experiment cookie "pp_experiment_product_data_source_test" with value "preview"
      And a registered RFA "p_product_experiment_v1":
        """
        function(context) { return context.product.name; }
        """
      When I request GET /product-experiment.html
      Then the response should contain "Preview Trail Shoe"
      And backend service "catalog-preview" should have received 1 request for GET /products/sku-123
      And backend service "catalog" should have received 0 requests for GET /products/sku-123
