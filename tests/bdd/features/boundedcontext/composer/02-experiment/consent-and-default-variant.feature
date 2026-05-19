Feature: Experiment consent and default variant
  As a product manager,
  I want users without experiment consent to receive the default variant without persistent assignment,
  so that rendering remains deterministic without tracking.

  Rule: If user does not allow any tracking, he should be assigned to a default variant that does not require tracking.

    Example: User with no tracking allowed
      Given a registered page config:
        """
        {
          "path": "/index.html",
          "page_id": "landing",
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
      And a registered RFA "cart-rfa":
        """
        function(context) { return "Rendered: Introduction is " + context.introduction; }
        """
      When I have not accepted any tracking and experiment cookies
      And I request GET /index.html
      Then the response should not contain a Cookie "pp_experiment_welcome_message_test" with value "variant_a" or "variant_b"

    Example: Experiment variant assigned but no experiment consent given
      Given a registered page config:
        """
        {
          "path": "/index.html",
          "page_id": "landing",
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
      And I have the experiment cookie "pp_experiment_welcome_message_test" with value "variant_b" without consenting to the experiment cookies
      And a registered RFA "cart-rfa":
        """
        function(context) { return "Rendered: Introduction is " + context.introduction; }
        """
      When I request GET /index.html
      Then the response should not contain a Cookie "pp_experiment_welcome_message_test" with value "variant_a" or "variant_b"
      And the response should delete the Cookie "pp_experiment_welcome_message_test"
      And the response should contain "Rendered: Introduction is Hello dear Customer"
