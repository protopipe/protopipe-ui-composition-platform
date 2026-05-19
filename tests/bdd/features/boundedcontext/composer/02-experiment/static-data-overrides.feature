Feature: Experiment static data overrides
  As a product manager,
  I want an experiment variant to override selected static page data,
  so that I can test content changes without adding variant logic to RFAs.

  Rule: If a user is assigned to an experiment variant with different static data, he should see the content defined for that variant.

    Scenario Outline: Experiment variant <variant_id> assigned content
      Given a registered page config:
        """
        {
          "path": "/index.html",
          "page_id": "landing",
          "type": "rfa",
          "template": "cart-v1",
          "rfa": "cart-rfa",
          "timeout_ms": 3000,
          "data": {
            "introduction": {
              "type": "static",
              "value": "Hello dear Customer"
            }
          }
        }
        """
      And a registered experiment:
        """
        {
          "experiment_id": "welcome_message_test",
          "variants": [
            {
              "id": "variant_a",
              "weight": 50
            },
            {
              "id": "variant_b",
              "weight": 50,
              "overrides": {
                "data": {
                  "introduction": {
                    "type": "static",
                    "value": "Hello Mate"
                  }
                }
              }
            }
          ]
        }
        """
      And I have accepted the experiment cookie "pp_experiment_welcome_message_test" with value "<variant_id>"
      And a registered RFA "cart-rfa":
        """
        function(context) { return "Rendered: Introduction is " + context.introduction; }
        """
      When I request GET /index.html
      Then the response should contain "Rendered: Introduction is <expected_greeting>"

      Examples:
        | variant_id | expected_greeting   |
        | variant_a  | Hello dear Customer |
        | variant_b  | Hello Mate          |
