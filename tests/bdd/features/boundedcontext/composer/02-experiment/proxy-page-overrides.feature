@WIP
Feature: Experiment Proxy Page overrides
  As a product manager,
  I want an experiment variant to route a page through an upstream proxy delivery mode,
  so that a monolith-backed page can be introduced as an A/B or canary experiment before it becomes stable Page configuration.

  Rule: Experiment variants may override the effective Page delivery mode.

    Scenario Outline: Experiment switches a page to Proxy Page delivery without marker replacement
      Given a registered page config:
        """
        {
          "path": "/shop/cart",
          "page_id": "cart",
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
      And an upstream monolith responds to GET /shop/cart with:
        """
        Legacy cart from monolith
        """
      And a registered experiment:
        """
        {
          "experiment_id": "cart_proxy_canary",
          "scope": {
            "path": "/shop/cart"
          },
          "variants": [
            {
              "id": "composer",
              "weight": 90
            },
            {
              "id": "proxy",
              "weight": 10,
              "overrides": {
                "delivery": {
                  "type": "upstream-proxy",
                  "origin": "http://legacy-monolith"
                }
              }
            }
          ]
        }
        """
      And I have accepted the experiment cookie "pp_experiment_cart_proxy_canary" with value "<variant_id>"
      And a registered RFA "cart-rfa":
        """
        function(context) { return "Composer cart in " + context.currency; }
        """
      When I request GET /shop/cart
      Then the response status should be 200
      And the response should contain "<expected_heading>"

      Examples:
        | variant_id | expected_heading           |
        | composer   | Composer cart in EUR       |
        | proxy      | Legacy cart from monolith  |

  Rule: A successful Proxy Page experiment can be promoted into Page configuration.
